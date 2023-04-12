CREATE TABLE completions (
	id SERIAL PRIMARY KEY, -- Simple ID
	achieved_mark INTEGER NOT NULL, -- The mark that was actually achieved
	total_marks INTEGER NOT NULL, -- The total marks available
	date DATE, -- The date of the completion
	comments TEXT, -- Any extra comments
	test_id INTEGER NOT NULL REFERENCES tests(id) -- The test that this completion belongs to
);
