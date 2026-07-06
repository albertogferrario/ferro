import { useForm, Link } from '@inertiajs/react'
import { Mail, Lock, ArrowRight, Loader2 } from 'lucide-react'
import { Button, Input, Label, Checkbox } from '@appolabs/ui'
import { AuthLayout } from '../../layouts'

interface LoginProps {
  errors?: Record<string, string[]>
}

export default function Login({ errors }: LoginProps) {
  const { data, setData, post, processing } = useForm({
    email: '',
    password: '',
    remember: false,
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    post('/login')
  }

  return (
    <AuthLayout
      title="Welcome back"
      subtitle="Sign in to your account to continue"
    >
      <form onSubmit={handleSubmit} className="space-y-4">
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
          <div className="flex items-center justify-between">
            <Label htmlFor="password">Password</Label>
            <Link
              href="/forgot-password"
              className="text-sm text-muted-foreground hover:text-primary"
            >
              Forgot password?
            </Link>
          </div>
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="password"
              type="password"
              placeholder="Enter your password"
              value={data.password}
              onChange={(e) => setData('password', e.target.value)}
              className="pl-10"
              autoComplete="current-password"
              required
            />
          </div>
          {errors?.password && (
            <p className="text-sm text-destructive">{errors.password[0]}</p>
          )}
        </div>

        <div className="flex items-center space-x-2">
          <Checkbox
            id="remember"
            checked={data.remember}
            onCheckedChange={(checked) => setData('remember', checked as boolean)}
          />
          <Label htmlFor="remember" className="text-sm font-normal cursor-pointer">
            Remember me
          </Label>
        </div>

        <Button type="submit" className="w-full" disabled={processing}>
          {processing ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Signing in...
            </>
          ) : (
            <>
              Sign in
              <ArrowRight className="ml-2 h-4 w-4" />
            </>
          )}
        </Button>
      </form>

      <div className="mt-6 text-center text-sm text-muted-foreground">
        Don't have an account?{' '}
        <Link href="/register" className="text-primary hover:underline">
          Sign up
        </Link>
      </div>
    </AuthLayout>
  )
}
