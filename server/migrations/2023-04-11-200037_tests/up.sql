CREATE TABLE tests (
	id SERIAL PRIMARY KEY, -- Simple ID for completions to reference
	subject TEXT NOT NULL, -- Maths, English, Science, etc.
	topic TEXT, -- Statistics, Shakespeare, Organic Chemistry, etc.
	date_or_id TEXT NOT NULL, -- Monday 3 June 2019, Mock Set 1, etc.
	qualification_level TEXT, -- GCSE, A Level, etc.
	exam_board TEXT, -- Edexcel, AQA, OCR, etc.
	paper_link TEXT, -- A link to the paper
	mark_scheme_link TEXT, -- A link to the mark scheme
	comments TEXT, -- Any extra comments
	user_id TEXT NOT NULL REFERENCES users(id), -- The user that owns this past paper
	-- A uniqueness constraint across all the fields means that we only exclude
	-- exact duplicates; changing any single field will allow an almost-copy to
	-- be inserted
	UNIQUE (id, subject, topic, date_or_id, qualification_level, exam_board, paper_link, mark_scheme_link, comments, user_id)
);
