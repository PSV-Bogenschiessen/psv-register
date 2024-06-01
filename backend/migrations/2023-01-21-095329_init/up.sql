CREATE TABLE "archers" (
	"bib"	INTEGER NOT NULL UNIQUE,
	"session"	INTEGER NOT NULL,
	"division"	TEXT NOT NULL,
	"class"	TEXT NOT NULL,
	"target"	TEXT NOT NULL,
	"individual qualification"	INTEGER NOT NULL,
	"team qualification"	INTEGER NOT NULL,
	"individual final"	INTEGER NOT NULL,
	"team final"	INTEGER NOT NULL,
	"mixed team final"	INTEGER NOT NULL,
	"last name"	TEXT NOT NULL,
	"first name"	TEXT NOT NULL,
	"gender"	INTEGER,
	"country code"	TEXT NOT NULL,
	"country name"	TEXT NOT NULL,
	"date of birth"	TEXT NOT NULL,
	"subclass"	TEXT,
	"country code 2"	TEXT,
	"country name 2"	TEXT,
	"country code 3"	TEXT,
	"country name 3"	TEXT,
	PRIMARY KEY("bib" AUTOINCREMENT)
);
CREATE TABLE "archer_additions" (
	"bib"	INTEGER NOT NULL REFERENCES archers(bib),
	"email"	TEXT,
	"comment"	TEXT,
	PRIMARY KEY("bib")
);
