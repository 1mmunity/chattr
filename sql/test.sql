INSERT INTO users (username, email, password)
  VALUES ('Adriel', 'mock@test.com', 'a');

--@block
INSERT INTO rooms.rooms (name, owner_id)
  VALUES ('bruh group', 1);

--@block
INSERT INTO rooms.messages (content, author_id, room_id)
  VALUES ('nice', 1, 1);

-- @block
SELECT * FROM rooms.messages;

--@block
DELETE FROM rooms.messages WHERE id=2 RETURNING *;

-- @block
SELECT * FROM rooms.rooms WHERE id=1;