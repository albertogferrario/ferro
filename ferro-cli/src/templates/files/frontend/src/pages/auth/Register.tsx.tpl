import { useForm, Link } from '@inertiajs/react'
import { User, Mail, Lock, ArrowRight, Loader2 } from 'lucide-react'
import { Button, Input, Label } from '@appolabs/ui'
import { AuthLayout } from '../../layouts'

interface RegisterProps {
  errors?: Record<string, string[]>
}

export default function Register({ errors }: RegisterProps) {
  const { data, setData, post, processing } = useForm({
    name: '',
    email: '',
    password: '',
    password_confirmation: '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    post('/register')
  }

  return (
    <AuthLayout
      title="Create an account"
      subtitle="Enter your information to get started"
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="name">Name</Label>
          <div className="relative">
            <User className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="name"
              type="text"
              placeholder="Your full name"
              value={data.name}
              onChange={(e) => setData('name', e.target.value)}
              className="pl-10"
              autoComplete="name"
              required
            />
          </div>
          {errors?.name && (
            <p className="text-sm text-destructive">{errors.name[0]}</p>
          )}
        </div>

        <div className="space-y-2">
          <Label htmlFor="email">Email</Label>
          <div className="relative">
            <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="email"
              type="email"
              placeholder="name@example.com"
              value={data.email}
              onChange={(e) => setData('email', e.target.value)}
              className="pl-10"
              autoComplete="email"
              required
            />
          </div>
          {errors?.email && (
            <p className="text-sm text-destructive">{errors.email[0]}</p>
          )}
        </div>

        <div className="space-y-2">
          <Label htmlFor="password">Password</Label>
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="password"
              type="password"
              placeholder="Minimum 8 characters"
              value={data.password}
              onChange={(e) => setData('password', e.target.value)}
              className="pl-10"
              autoComplete="new-password"
              required
            />
          </div>
          {errors?.password && (
            <p className="text-sm text-destructive">{errors.password[0]}</p>
          )}
        </div>

        <div className="space-y-2">
          <Label htmlFor="password_confirmation">Confirm Password</Label>
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="password_confirmation"
              type="password"
              placeholder="Repeat your password"
              value={data.password_confirmation}
              onChange={(e) => setData('password_confirmation', e.target.value)}
              className="pl-10"
              autoComplete="new-password"
              required
            />
          </div>
          {errors?.password_confirmation && (
            <p className="text-sm text-destructive">{errors.password_confirmation[0]}</p>
          )}
        </div>

        <Button type="submit" className="w-full" disabled={processing}>
          {processing ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Creating account...
            </>
          ) : (
            <>
              Create account
              <ArrowRight className="ml-2 h-4 w-4" />
            </>
          )}
        </Button>
      </form>

      <div className="mt-6 text-center text-sm text-muted-foreground">
        Already have an account?{' '}
        <Link href="/login" className="text-primary hover:underline">
          Sign in
        </Link>
      </div>
    </AuthLayout>
  )
}
