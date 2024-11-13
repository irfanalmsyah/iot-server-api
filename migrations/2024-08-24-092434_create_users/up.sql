-- Your SQL goes here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR (255) NOT NULL UNIQUE, 
    email VARCHAR (255) NOT NULL UNIQUE, 
    password VARCHAR (255) NOT NULL, 
    status BOOLEAN DEFAULT FALSE, 
    isadmin BOOLEAN DEFAULT FALSE
)