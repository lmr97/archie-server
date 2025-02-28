-- CREATE DATABASE archie;
USE archie;

CREATE TABLE guestbook
(
    id              INT NOT NULL AUTO_INCREMENT,
    dateSubmitted   DATETIME,
    guestName       VARCHAR(30)  NOT NULL,
    guestNote       VARCHAR(300),
    PRIMARY KEY     (id) 
);

CREATE TABLE hitLog
(
    id              INT NOT NULL AUTO_INCREMENT,
    hitTime         DATETIME,
    userAgent       VARCHAR(150),
    PRIMARY KEY     (id) 
);