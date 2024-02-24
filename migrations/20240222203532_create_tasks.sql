create table tasks (
    id uuid default gen_random_uuid() primary key,
    story_id uuid references stories(id) not null,
    name varchar(100) not null,
    status varchar(100) not null default 'incomplete',
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

create index tasks_story_id_index ON tasks USING btree(story_id);
