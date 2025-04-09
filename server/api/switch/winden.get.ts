import { generateSlug } from 'random-word-slugs';
import { customAlphabet, nanoid } from 'nanoid';
import { numbers as numDict } from 'nanoid-dictionary';
import { isNull } from 'drizzle-orm';

export default defineEventHandler(async (event) => {
    const query = getQuery(event)

    let licenseKey: string = query.license as string;

    try {
        const response = await fetch('https://dplugins.com', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
            },
            body: new URLSearchParams({
                edd_action: 'activate_license',
                license: licenseKey,
                item_id: '46608',
                url: `https://${generateSlug(2)}-${customAlphabet(numDict + 'abcdef', 6)()}.instawp.xyz`,
                nonce: nanoid(10)
            }).toString()
        });
        const data = await response.json();

        if (!data.success || data.license !== 'valid' || data.item_id !== 46608) {
            throw new Error('Failed to validate license. License may be invalid');
        }
    } catch (error) {
        return {
            error: 'Failed to validate license. License may be invalid'
        }
    }

    const windenLicense = await useDrizzle().query.windenLicenses.findFirst({
        where: (w) => eq(w.licenseKey, licenseKey),
        with: {
            discount: true,
        }
    });

    if (windenLicense) {
        return {
            data: {
                license_key: windenLicense.licenseKey,
                discount_code: windenLicense.discount.code,
                claimed_at: windenLicense.claimedAt,
                campain: windenLicense.discount.campaign,
            }
        }
    }

    const discount = await useDrizzle()
        .select()
        .from(tables.discounts)
        .leftJoin(tables.windenLicenses, eq(tables.discounts.id, tables.windenLicenses.discountId))
        .where(
            and(
                isNull(tables.windenLicenses.id), // No relation exists
                eq(tables.discounts.campaign, 'Bye-Winden') // Campaign is 'Bye-Winden'
            )
        )
        .limit(1); // Get only the first result

    if (!discount || discount.length === 0) {
        return {
            error: 'No available discounts'
        }
    }

    // insert the licenseKey into the database with the discountId
    const insertedWindenLicense = await useDrizzle().insert(tables.windenLicenses).values({
        licenseKey: licenseKey,
        discountId: discount[0].discounts.id,
        claimedAt: sql`(unixepoch())`,
    }).returning();

    return {
        data: {
            license_key: insertedWindenLicense[0].licenseKey,
            discount_code: discount[0].discounts.code,
            claimed_at: insertedWindenLicense[0].claimedAt,
            campain: discount[0].discounts.campaign,
        }
    }
})