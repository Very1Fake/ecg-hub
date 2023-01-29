/* Triggers */

drop trigger if exists updated_at_trigger on "User";
drop trigger if exists updated_at_trigger on "WebSession";
drop trigger if exists updated_at_trigger on "GameSession";
drop trigger if exists updated_at_trigger on "MobileSession";

/* Functions */

drop function if exists updated_at_time_func;

/* Tables */

drop table "MobileSession";
drop table "GameSession";
drop table "WebSession";
drop table "User";

-- Addtional type
drop domain if exists email;

-- Extensions
drop extension if exists "uuid-ossp";
drop extension if exists citext;
