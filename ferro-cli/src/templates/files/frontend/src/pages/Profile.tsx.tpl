import { useForm, usePage } from '@inertiajs/react'
import { User, Mail, Loader2, Save } from 'lucide-react'
import {
  Button,
  Input,
  Label,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Avatar,
  AvatarFallback,
  Separator,
} from '@appolabs/ui'
import { AppLayout } from '../layouts'
import type { User as UserType } from '../types/inertia-props'

interface ProfileProps {
  errors?: Record<string, string[]>
}

export default function Profile({ errors }: ProfileProps) {
  const { props } = usePage()
  const auth = props.auth as { user: UserType | null } | undefined
  const user = auth?.user

  const { data, setData, put, processing } = useForm({
    name: user?.name || '',
    email: user?.email || '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    put('/profile')
  }

  if (!user) return null

  return (
    <AppLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Profile</h1>
          <p className="text-muted-foreground">
            Manage your account settings and preferences.
          </p>
        </div>

        <div className="grid gap-6">
          {/* Profile Information */}
          <Card>
            <CardHeader>
              <CardTitle>Profile Information</CardTitle>
              <CardDescription>
                Update your account's profile information and email address.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmit} className="space-y-6">
                {/* Avatar section */}
                <div className="flex items-center gap-4">
                  <Avatar className="h-20 w-20">
                    <AvatarFallback className="text-2xl bg-primary/10 text-primary">
                      {user.name.charAt(0).toUpperCase()}
                    </AvatarFallback>
                  </Avatar>
                  <div>
                    <h3 className="font-medium">{user.name}</h3>
                    <p className="text-sm text-muted-foreground">{user.email}</p>
                  </div>
                </div>

                <Separator />

                <div className="grid gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="name">Name</Label>
                    <div className="relative">
                      <User className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                      <Input
                        id="name"
                        type="text"
                        value={data.name}
                        onChange={(e) => setData('name', e.target.value)}
                        className="pl-10"
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
                        value={data.email}
                        onChange={(e) => setData('email', e.target.value)}
                        className="pl-10"
                        required
                      />
                    </div>
                    {errors?.email && (
                      <p className="text-sm text-destructive">{errors.email[0]}</p>
                    )}
                  </div>
                </div>

                <div className="flex justify-end">
                  <Button type="submit" disabled={processing}>
                    {processing ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Saving...
                      </>
                    ) : (
                      <>
                        <Save className="mr-2 h-4 w-4" />
                        Save changes
                      </>
                    )}
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>

          {/* Change Password */}
          <Card>
            <CardHeader>
              <CardTitle>Update Password</CardTitle>
              <CardDescription>
                Ensure your account is using a secure password.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ChangePasswordForm />
            </CardContent>
          </Card>
        </div>
      </div>
    </AppLayout>
  )
}

function ChangePasswordForm() {
  const { data, setData, put, processing, reset, errors } = useForm({
    current_password: '',
    password: '',
    password_confirmation: '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    put('/profile/password', {
      onSuccess: () => reset(),
    })
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="current_password">Current Password</Label>
        <Input
          id="current_password"
          type="password"
          value={data.current_password}
          onChange={(e) => setData('current_password', e.target.value)}
          required
        />
        {errors?.current_password && (
          <p className="text-sm text-destructive">{errors.current_password}</p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="password">New Password</Label>
        <Input
          id="password"
          type="password"
          value={data.password}
          onChange={(e) => setData('password', e.target.value)}
          required
        />
        {errors?.password && (
          <p className="text-sm text-destructive">{errors.password}</p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="password_confirmation">Confirm New Password</Label>
        <Input
          id="password_confirmation"
          type="password"
          value={data.password_confirmation}
          onChange={(e) => setData('password_confirmation', e.target.value)}
          required
        />
        {errors?.password_confirmation && (
          <p className="text-sm text-destructive">{errors.password_confirmation}</p>
        )}
      </div>

      <div className="flex justify-end">
        <Button type="submit" disabled={processing}>
          {processing ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Updating...
            </>
          ) : (
            'Update Password'
          )}
        </Button>
      </div>
    </form>
  )
}
