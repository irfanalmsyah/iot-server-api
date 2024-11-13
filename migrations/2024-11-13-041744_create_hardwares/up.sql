-- Your SQL goes here
CREATE TABLE hardwares (
    id SERIAL PRIMARY KEY,
    name VARCHAR (255) NOT NULL,
    type VARCHAR (255) NOT NULL,
    description TEXT NOT NULL
);