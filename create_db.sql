CREATE DATABASE archie;
USE archie;

SET TIME_ZONE='+00:00';

CREATE TABLE guestbook
(
    id              INT NOT NULL AUTO_INCREMENT,
    dateSubmitted   DATETIME,
    guestName       VARCHAR(50)  NOT NULL,
    guestNote       VARCHAR(1000),
    PRIMARY KEY     (id) 
);

CREATE TABLE hitLog
(
    id              INT NOT NULL AUTO_INCREMENT,
    hitTime         TIMESTAMP,
    userAgent       VARCHAR(150),
    PRIMARY KEY     (id) 
);