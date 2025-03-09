-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- users table 
CREATE TABLE 
    IF NOT EXISTS users (
        user_id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
        user_name VARCHAR(50) NOT NULL UNIQUE,
        user_first_name VARCHAR(50) NOT NULL,
        user_last_name VARCHAR(50) NOT NULL,
        user_email VARCHAR(250) NOT NULL UNIQUE, 
        user_password TEXT NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

-- records table
CREATE TABLE 
    IF NOT EXISTS records (
        record_id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
        artist VARCHAR(100) NOT NULL,
        title VARCHAR(100) NOT NULL,
        released DATE NOT NULL,
        genre TEXT[],
        format VARCHAR(50),
        price DECIMAL(10,2),
        label VARCHAR(150) NOT NULL,
        duration_length TIME NOT NULL
    );

-- record_stores table
CREATE TABLE 
    IF NOT EXISTS record_stores (
        record_store_id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
        store_name VARCHAR(50) NOT NULL,
        store_address VARCHAR(200) NOT NULL,
        store_city VARCHAR(75) NOT NULL,
        store_state VARCHAR(50) NOT NULL,
        store_zip VARCHAR(20) NOT NULL, 
        phone_number VARCHAR(20), 
        website TEXT
    );


-- user_record_stores table
CREATE TABLE
    IF NOT EXISTS user_record_stores (
        user_favorite_stores_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        user_key UUID NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
        record_store_id UUID NOT NULL REFERENCES record_stores (record_store_id) ON DELETE CASCADE
    );

-- user_records table
CREATE TABLE 
    IF NOT EXISTS user_records (
    user_record_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    record_id UUID NOT NULL REFERENCES records(record_id) ON DELETE CASCADE,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- user_wish_list table
CREATE TABLE 
    IF NOT EXISTS user_wishlist (
        user_wish_list_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
        record_id UUID NOT NULL REFERENCES records(record_id) ON DELETE CASCADE,
        added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );


-- populate data

-- create an initial user myself 
INSERT INTO users (user_id, user_name, user_first_name, user_last_name, user_email, user_password)
VALUES 
    (uuid_generate_v4(), 'goodguygregory', 'Greg', 'Witt', 'greg@goodguygregory.com', '$2b$12$P/Q7.Bv5gcyMgxkHRhHpmuzmnHhM5XMAot/Grjn/sbmlFKtrcZjQa');

-- -- create some basic records you should check out. 
INSERT INTO records (record_id, artist, title, released, genre, format, price, label, duration_length)
VALUES 
    (uuid_generate_v4(),'Thievery Corporation', 'The Temple of I & I', '2017-02-10', ARRAY['Dub', 'Trip Hop', 'Electronica'], 'LP', 30.00, 'ESL Music', '01:00:00' ),
    (uuid_generate_v4(),'Khruangbin', 'The Universe Smiles Upon You', '2015-10-06', ARRAY['Dub', 'Psychedelic Rock', 'Surf Rock', 'Funk' ], 'LP', 21.00, 'Night Time Stories', '00:39:44'),
    (uuid_generate_v4(),'Tame Impala', 'Currents', '2015-07-17', ARRAY['Psychedelic Pop', 'Synth-Pop', 'Electronica'], 'LP', 28.00, 'Modular', '00:51:12'), 
    (uuid_generate_v4(),'Bonobo', 'Migration', '2017-01-13', ARRAY['Electronica', 'Ambient', 'Downtempo'], 'LP', 27.00, 'Ninja Tune', '01:01:53'),
    (uuid_generate_v4(),'The Breathing Effect', 'Photosynthesis', '2020-03-27', ARRAY['Psychedelic Rock', 'Electronica', 'Jazz Fusion'], 'LP', 25.00, 'Alpha Pup Records', '00:45:05'),
    (uuid_generate_v4(),'Herbie Hancock', 'Head Hunters', '1973-10-26', ARRAY['Jazz-Funk', 'Jazz Fusion'], 'LP', 37.00, 'Columbia', '00:41:52' ),
    (uuid_generate_v4(),'Boards of Canada', 'Music Has The Right To Children', '1998-04-20', ARRAY['Electronic', 'Trip Hop', 'Ambient'], 'LP', 21.00, 'Warp Records', '01:11:00' ),
    (uuid_generate_v4(),'Röyksopp', 'Melody A.M.', '2001-08-22', ARRAY['Synth-Pop', 'Trip Hop', 'Electronica'], 'LP', 21.00, 'Wall of Sound', '00:47:09'),
    (uuid_generate_v4(),'Tom Misch', 'Beat Tape 2', '2015-08-28', ARRAY['Jazz Funk', 'R&B', 'Electronica'], 'LP', 56.00, 'Beyond The Groove', '00:49:19' ),
    (uuid_generate_v4(),'Populous', 'Stasi', '2021-06-11', ARRAY['Dub', 'Trip Hop', 'Electronica'], 'LP', 21.00, 'La Tempesta Dischi', '00:42:49' ),
    (uuid_generate_v4(),'Four Tet', 'New Energy', '2017-09-29', ARRAY['Indie Electronic', 'Folktronica', 'House'], 'LP', 26.00, 'Text Records', '00:56:27');

-- create record_stores 
INSERT INTO record_stores (record_store_id, store_name, store_address, store_city, store_state, store_zip, phone_number, website)
VALUES
    (uuid_generate_v4(), 'Guestroom Records', '1806 Frankfort Ave', 'Louisville', 'KY','40206', '502-883-0478', 'https://www.guestroomrecordslouisville.com/'),    
    (uuid_generate_v4(), 'Music Millennium','3158 E Burnside St', 'Portland', 'OR','97214', '503-231-8926','https://www.musicmillennium.com' ),    
    (uuid_generate_v4(), 'Harvest Records', '415 Haywood Rd, Suite B', 'Asheville', 'NC','28806','828-258-2999', 'https://www.harvest-records.com/');    

