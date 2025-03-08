-- Add down migration script here

-- delete the users table
DROP TABLE IF EXISTS users;

-- delete the records table
DROP TABLE IF EXISTS records;

-- delete the record-stores table
DROP TABLE IF EXISTS record_stores;

-- delete the user_record_stores table 
DROP TABLE IF EXISTS user_record_stores;

-- delete the user_records table
DROP TABLE IF EXISTS user_records;

-- delete the user_wishlist table
DROP TABLE IF EXISTS user_wishlist;
