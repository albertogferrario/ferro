import { useForm, Link } from '@inertiajs/react'
import { ArrowLeft, ArrowRight, Loader2, CheckCircle } from 'lucide-react'
import { Button, Input, Label } from '@appolabs/ui'
import { AuthLayout } from '../../layouts'

interface ForgotPasswordProps {
  errors?: Record<string, string[]>
  status?: string | null
}

export default function ForgotPassword({ errors, status }: ForgotPasswordProps) {
  const { data, setData, post, processing } = useForm({
    email: '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    post('/forgot-password')
  }

  return (
    <AuthLayout
      title="Forgot password"
      subtitle="Enter your email and we'll send you a reset link"
    >
      {status ? (
        <div className="space-y-6">
          <div className="flex flex-col items-center gap-4 text-center">
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
              <CheckCircle className="h-6 w-6 text-primary" />
            </div>
            <div>
              <h3 className="font-semibold text-foreground">Check your email</h3>
              <p className="mt-1 text-sm text-muted-foreground">{status}</p>
            </div>
          </div>

          <div className="text-center">
            <Link
              href="/login"
              className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
            >
              <ArrowLeft className="h-4 w-4" />
              Back to login
            </Link>
          </div>
        </div>
      ) : (
        <>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="name@example.com"
                value={data.email}
                onChange={(e) => setData('email', e.target.value)}
                autoComplete="email"
                required
              />
              {errors?.email && (
                <p className="text-sm text-destructive">{errors.email[0]}</p>
              )}
            </div>

            <Button type="submit" className="w-full" disabled={processing}>
              {processing ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Sending...
                </>
              ) : (
                <>
                  Send reset link
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
        </>
      )}
    </AuthLayout>
  )
}
