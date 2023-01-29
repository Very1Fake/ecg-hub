-- Extension initialization
create extension if not exists "uuid-ossp";
create extension if not exists citext;

-- Additional type
create domain email as citext
  check ( value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );

/* Table Creation */

create table "User" (
	uuid uuid primary key default uuid_generate_v4(),
	username varchar(24) unique not null,
	email email unique not null,
	password varchar(256) not null,
	other jsonb not null default '{}'::jsonb,
	status smallint not null default 1,
	updated_at timestamptz not null default now(),
	created_at timestamptz not null default now()
);

create table "WebSession" (
	uuid uuid primary key default uuid_generate_v4(),
	sub uuid unique not null references "User" on delete cascade on update cascade,
	token uuid unique not null default uuid_generate_v4(),
	exp timestamptz not null,
	updated_at timestamptz not null default now(),
	created_at timestamptz not null default now()
);

create table "GameSession" (
	uuid uuid primary key default uuid_generate_v4(),
	sub uuid unique not null references "User" on delete cascade on update cascade,
	token uuid unique not null default uuid_generate_v4(),
	exp timestamptz not null,
	updated_at timestamptz not null default now(),
	created_at timestamptz not null default now()
);

create table "MobileSession" (
	uuid uuid primary key default uuid_generate_v4(),
	sub uuid unique not null references "User" on delete cascade on update cascade,
	token uuid unique not null default uuid_generate_v4(),
	exp timestamptz not null,
	updated_at timestamptz not null default now(),
	created_at timestamptz not null default now()
);

/* Functions */

create or replace function updated_at_time_func() returns trigger as
$$
begin
	new.updated_at = now();
	return new;
end;
$$ language plpgsql;

/* Tiggers */

drop trigger if exists updated_at_trigger on "User";
create trigger updated_at_trigger
	before update on "User"
	for each row
execute function updated_at_time_func();

drop trigger if exists updated_at_trigger on "WebSession";
create trigger updated_at_trigger
	before update on "WebSession"
	for each row
execute function updated_at_time_func();

drop trigger if exists updated_at_trigger on "GameSession";
create trigger updated_at_trigger
	before update on "GameSession"
	for each row
execute function updated_at_time_func();

drop trigger if exists updated_at_trigger on "MobileSession";
create trigger updated_at_trigger
	before update on "MobileSession"
	for each row
execute function updated_at_time_func();

/* Reserver accounts */

insert into "User" (uuid, username, email, password, created_at) values
	('00000000-0000-ffff-0000-000000000001', 'server', 'server@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000002', 'admin', 'admin@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000003', 'broadcast', 'broadcast@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000004', 'console', 'console@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000005', 'notify', 'notify@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000006', 'example', 'example@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000007', 'blacklist', 'blacklist@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000008', 'blocklist', 'blocklist@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-000000000009', 'whitelist', 'whitelist@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-00000000000a', 'session', 'session@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-00000000000b', 'web', 'web@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-00000000000d', 'mobile', 'mobile@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-00000000000e', 'game', 'game@example.com', 'nopass', 'epoch'),
	('00000000-0000-ffff-0000-00000000000f', 'block', 'block@example.com', 'nopass', 'epoch');