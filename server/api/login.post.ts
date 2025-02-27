import { z } from 'zod'

const bodySchema = z.object({
    email: z.string().email(),
    password: z.string()
})

export default defineEventHandler(async (event) => {
    const { email, password } = await readValidatedBody(event, bodySchema.parse)

    const user = await useDrizzle()
        .query.users.findFirst({
            where: eq(tables.users.email, email)
        })

    if (!user || !await verifyPassword(password, user.password)) {
        throw createError({
            statusCode: 401,
            message: 'Bad credentials'
        })
    }

    await setUserSession(event, {
        user: {
            id: user.id,
            name: user.name
        }
    })

    return {}
})