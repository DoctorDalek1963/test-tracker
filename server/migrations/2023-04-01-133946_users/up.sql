CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Function taken from https://stackoverflow.com/a/41988979/12985838
CREATE OR REPLACE FUNCTION generate_uid(size INT) RETURNS TEXT AS $$
DECLARE
	characters TEXT := 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	bytes BYTEA := gen_random_bytes(size);
	l INT := length(characters);
	i INT := 0;
	output TEXT := '';
BEGIN
	WHILE i < size LOOP
		output := output || substr(characters, get_byte(bytes, i) % l + 1, 1);
		i := i + 1;
	END LOOP;
	RETURN output;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE TABLE users (
	id TEXT PRIMARY KEY DEFAULT ('user_' || generate_uid(50)),
	username TEXT NOT NULL UNIQUE,
	hashed_password TEXT NOT NULL
);
