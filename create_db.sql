CREATE DATABASE archie;
USE archie;

CREATE TABLE guestbook
(
    id              INT NOT NULL AUTO_INCREMENT,
    dateSubmitted   DATETIME,
    guestName       VARCHAR(30)  NOT NULL,
    guestNote       VARCHAR(300),
    PRIMARY KEY     (id) 
);

CREATE TABLE hitsLog
(
    id              INT NOT NULL AUTO_INCREMENT,
    hitTime         DATETIME,
    PRIMARY KEY     (id) 
);