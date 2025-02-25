import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core'

export const users = sqliteTable('users', {
    id: integer('id').primaryKey({ autoIncrement: true }),
    name: text('name').notNull(),
    email: text('email').notNull().unique(),
    password: text('password').notNull(),
    createdAt: integer('created_at', { mode: 'timestamp' }).notNull(),
})

export const presets = sqliteTable('presets', {
    id: integer('id').primaryKey({ autoIncrement: true }),
    uid: text('uid', { length: 21 }).notNull().unique(),
    name: text('name').notNull(),
    userId: integer('user_id').notNull().references(() => users.id),
    createdAt: integer('created_at', { mode: 'timestamp' }).notNull(),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).notNull(),
    payload: text('payload').notNull(),
    blobPath: text('blob_path'),
})