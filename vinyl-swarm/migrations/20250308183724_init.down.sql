-- Add down migration script here

-- delete the users table
DROP TABLE IF EXISTS users CASCADE;

-- delete the records table
DROP TABLE IF EXISTS records CASCADE;

-- delete the record-stores table
DROP TABLE IF EXISTS record_stores CASCADE;

-- delete the user_record_stores table 
DROP TABLE IF EXISTS user_record_stores CASCADE;

-- delete the user_records table
DROP TABLE IF EXISTS user_records CASCADE;

-- delete the user_wishlist table
DROP TABLE IF EXISTS user_wishlist CASCADE;
