create table posts (
	id integer not null primary key autoincrement,
	title varchar not null,
	body text not null,
	published boolean not null default 0
);
