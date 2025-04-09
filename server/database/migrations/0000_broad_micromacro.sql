CREATE TABLE `discounts` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`discount_code` text NOT NULL,
	`campaign` text,
	`start_date` integer,
	`end_date` integer,
	`created_at` integer
);
--> statement-breakpoint
CREATE TABLE `winden_licenses` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`license_key` text NOT NULL,
	`discount_id` integer NOT NULL,
	`claimed_at` integer
);
