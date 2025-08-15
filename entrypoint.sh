#!/bin/sh

# Create the database file if it doesn't exist
if [ ! -f "db.sqlite" ]; then
    touch db.sqlite
fi

# Run migrations
sqlx database setup

# Start the application
./acortador-url
