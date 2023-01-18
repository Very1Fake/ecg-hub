create extension if not exists "uuid-ossp";
create extension if not exists citext;

create domain email as citext
  check ( value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );


create table "User" (
	uuid uuid primary key default uuid_generate_v4(),
	username varchar(24) unique not null,
	email email unique not null,
	password varchar(256) not null,
	other jsonb not null default '"{}"'::jsonb,
	status smallint not null default 1,
	updated_at timestamptz not null default now(),
	created_at timestamptz not null default now()
);

create table "WebSession" (
	sub uuid primary key references "User" on delete cascade on update cascade,
	uuid uuid not null default uuid_generate_v4(),
	exp timestamptz not null
);

create table "GameSession" (
	sub uuid primary key references "User" on delete cascade on update cascade,
	uuid uuid not null default uuid_generate_v4(),
	exp timestamptz not null
);

create table "MobileSession" (
	sub uuid primary key references "User" on delete cascade on update cascade,
	uuid uuid not null default uuid_generate_v4(),
	exp timestamptz not null
);