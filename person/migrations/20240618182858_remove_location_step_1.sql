-- Add migration script here
ALTER TABLE person
ALTER COLUMN location DROP NOT NULL;
