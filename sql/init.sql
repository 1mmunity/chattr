DROP TABLE IF EXISTS users CASCADE;
-- DANGEROUS! PLEASE CHECK THIS BEFORE RUNNING!

CREATE TABLE IF NOT EXISTS users (
  id SERIAL NOT NULL PRIMARY KEY,
  username VARCHAR(32) NOT NULL,
  email VARCHAR(254) NOT NULL UNIQUE,
  password VARCHAR(64) NOT NULL
);

CREATE SCHEMA IF NOT EXISTS rooms;

CREATE TABLE IF NOT EXISTS rooms.messages (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  room_id INT NOT NULL REFERENCES rooms.rooms (id),
  author_id INT NOT NULL REFERENCES users (id),
  content TEXT NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS rooms.rooms (
  id SERIAL NOT NULL PRIMARY KEY,
  owner_id INT NOT NULL REFERENCES users (id),
  name VARCHAR(255) NOT NULL,
  description TEXT,
  created_at TIMESTAMP WITH TImE ZONE DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS rooms.members (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  room_id INT NOT NULL REFERENCES rooms.rooms (id) ON DELETE CASCADE,
  user_id INT NOT NULL REFERENCES users (id) ON DELETE CASCADE
);
