-- Add migration script here
CREATE TABLE IF NOT EXISTS blog_spots (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL
)