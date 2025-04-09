import { relations } from 'drizzle-orm'
import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core'

export const discounts = sqliteTable('discounts', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  code: text('discount_code').notNull(),
  campaign: text('campaign'),
  startDate: integer('start_date', { mode: 'timestamp' }),
  endDate: integer('end_date', { mode: 'timestamp' }),
  createdAt: integer('created_at', { mode: 'timestamp' }),
})

export const windenLicenses = sqliteTable('winden_licenses', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  licenseKey: text('license_key').notNull(),
  discountId: integer('discount_id').notNull(),
  claimedAt: integer('claimed_at', { mode: 'timestamp' }),
})

export const windenLicenseRelations = relations(windenLicenses, ({ one }) => ({
  discount: one(discounts, {
    fields: [windenLicenses.discountId],
    references: [discounts.id],
  }),
}))

