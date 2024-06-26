create type view_format_version as enum ('v1');

create table view
(
    view_id             uuid primary key default uuid_generate_v1mc(),
    CONSTRAINT "tabular_ident_fk" FOREIGN KEY (view_id) REFERENCES tabular (tabular_id),
    view_format_version view_format_version not null,
    -- Speed up S3 Signing requests. Otherwise not needed
    -- as the location is stored in the metadata.
    location            text                not null,
    metadata_location   text                not null
);
call add_time_columns('"view"');
select trigger_updated_at('"view"');

create table view_schema
(
    schema_id int primary key,
    view_id   uuid  not null REFERENCES view (view_id) ON DELETE CASCADE,
    -- the schema object is quite complex and I'm not sure about the benefits of inviting that complexity into sql
    schema    jsonb not null
);
call add_time_columns('view_schema');
select trigger_updated_at('view_schema');

create table view_version
(
    view_version_uuid uuid        not null primary key default uuid_generate_v1mc(),
    view_id           uuid        not null REFERENCES view (view_id) ON DELETE CASCADE,
    version_id        bigint      not null,
    schema_id         int         not null REFERENCES view_schema (schema_id) ON DELETE CASCADE,
    timestamp         timestamptz not null,
    constraint "unique_version_per_metadata" unique (view_id, version_id),
    constraint "unique_version_per_metadata_including_pkey" unique (view_version_uuid, view_id, version_id)
);
call add_time_columns('view_version');
select trigger_updated_at('view_version');

create table view_properties
(
    property_id uuid primary key default uuid_generate_v1mc(),
    view_id     uuid not null REFERENCES view (view_id) ON DELETE CASCADE,
    key         text not null,
    value       text not null
);


call add_time_columns('view_properties');
select trigger_updated_at('"view_properties"');


create table current_view_metadata_version
(
    view_id      uuid primary key REFERENCES view (view_id) ON DELETE CASCADE,
    version_uuid uuid not null REFERENCES view_version (view_version_uuid) ON DELETE CASCADE,
    version_id   int8 not null,
    FOREIGN KEY (version_uuid, view_id, version_id) REFERENCES view_version (view_version_uuid, view_id, version_id)
);

call add_time_columns('current_view_metadata_version');
select trigger_updated_at('"current_view_metadata_version"');

create table view_version_log
(
    id         uuid primary key default uuid_generate_v1mc(),
    view_id    uuid        not null,
    version_id bigint      not null,
    timestamp  timestamptz not null,
    FOREIGN KEY (view_id, version_id) REFERENCES view_version (view_id, version_id) ON DELETE CASCADE
);
call add_time_columns('view_version_log');

create table metadata_summary
(
    summary_tuple_id uuid primary key default uuid_generate_v1mc(),
    version_id       uuid not null REFERENCES view_version (view_version_uuid) ON DELETE CASCADE,
    key              text not null,
    value            text not null
);

call add_time_columns('metadata_summary');
select trigger_updated_at('"metadata_summary"');

create type view_representation_type as enum ('sql');
create table view_representation
(
    view_representation_id uuid primary key default uuid_generate_v1mc(),
    view_id                uuid                     not null REFERENCES view (view_id) ON DELETE CASCADE,
    view_version_uuid      uuid                     not null REFERENCES view_version (view_version_uuid) ON DELETE CASCADE,
    typ                    view_representation_type not null,
    sql                    text                     not null,
    dialect                text                     not null
);

call add_time_columns('view_representation');
select trigger_updated_at('"view_representation"');
