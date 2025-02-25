import { z } from 'zod'

const bodySchema = z.object({
  email: z.string().email(),
  password: z.string(),
  name: z.string()
})

export default defineEventHandler(async (event) => {
  const { email, password, name } = await readValidatedBody(event, bodySchema.parse)

  const existingUser = await useDrizzle().query.users.findFirst({
    where: eq(tables.users.email, email)
  })

  if (existingUser) {
    throw createError({
      statusCode: 400,
      message: 'User already exists',
      data: {
        issues: [{
          path: ['email'],
          message: 'User already exists'
        }]
      }
    })
  }

  const passwordHash = await hashPassword(password)

  const user = await useDrizzle().insert(tables.users).values({
    email,
    password: passwordHash,
    name,
    createdAt: new Date()
  }).returning().get()

  return {}
})