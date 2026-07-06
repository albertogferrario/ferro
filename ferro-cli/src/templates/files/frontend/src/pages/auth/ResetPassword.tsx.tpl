import { useForm, Link } from '@inertiajs/react'
import { ArrowLeft, ArrowRight, Loader2 } from 'lucide-react'
import { Button, Input, Label } from '@appolabs/ui'
import { AuthLayout } from '../../layouts'

interface ResetPasswordProps {
  email: string
  token: string
  errors?: Record<string, string[]>
}

export default function ResetPassword({ email, token, errors }: ResetPasswordProps) {
  const { data, setData, post, processing } = useForm({
    email,
    token,
    password: '',
    password_confirmation: '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    post('/reset-password')
  }

  return (
    <AuthLayout
      title="Reset password"
      subtitle="Choose a new password for your account"
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <input type="hidden" name="email" value={data.email} />
        <input type="hidden" name="token" value={data.token} />

        {errors?.token && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-3">
            <p className="text-sm text-destructive">{errors.token[0]}</p>
          </div>
        )}

        <div className="space-y-2">
          <Label htmlFor="password">New password</Label>
          <Input
            id="password"
            type="password"
            placeholder="Minimum 8 characters"
            value={data.password}
            onChange={(e) => setData('password', e.target.value)}
            autoComplete="new-password"
            required
          />
          {errors?.password && (
            <p className="text-sm text-destructive">{errors.password[0]}</p>
          )}
        </div>

        <div className="space-y-2">
          <Label htmlFor="password_confirmation">Confirm password</Label>
          <Input
            id="password_confirmation"
            type="password"
            placeholder="Repeat your password"
            value={data.password_confirmation}
            onChange={(e) => setData('password_confirmation', e.target.value)}
            autoComplete="new-password"
            required
          />
          {errors?.password_confirmation && (
            <p className="text-sm text-destructive">{errors.password_confirmation[0]}</p>
          )}
        </div>

        <Button type="submit" className="w-full" disabled={processing}>
          {processing ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Resetting...
            </>
          ) : (
            <>
              Reset password
              <ArrowRight className="ml-2 h-4 w-4" />
            </>
          )}
        </Button>
      </form>

      <div className="mt-6 text-center">
        <Link
          href="/login"
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to login
        </Link>
      </div>
    </AuthLayout>
  )
}
