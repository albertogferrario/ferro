import { usePage } from '@inertiajs/react'
import { Activity, Users, FileText, TrendingUp } from 'lucide-react'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@appolabs/ui'
import { AppLayout } from '../layouts'
import type { User } from '../types/inertia-props'

interface StatCardProps {
  title: string
  value: string
  description: string
  icon: React.ReactNode
}

function StatCard({ title, value, description, icon }: StatCardProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <div className="h-4 w-4 text-muted-foreground">{icon}</div>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  )
}

export default function Dashboard() {
  const { props } = usePage()
  const auth = props.auth as { user: User | null } | undefined
  const user = auth?.user

  if (!user) return null

  return (
    <AppLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            Welcome back, {user.name}!
          </h1>
          <p className="text-muted-foreground">
            Here's an overview of your account.
          </p>
        </div>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <StatCard
            title="Total Views"
            value="2,350"
            description="+20.1% from last month"
            icon={<Activity className="h-4 w-4" />}
          />
          <StatCard
            title="Active Users"
            value="1,234"
            description="+180 new users"
            icon={<Users className="h-4 w-4" />}
          />
          <StatCard
            title="Documents"
            value="573"
            description="+12 since last week"
            icon={<FileText className="h-4 w-4" />}
          />
          <StatCard
            title="Growth"
            value="+12.5%"
            description="Compared to last quarter"
            icon={<TrendingUp className="h-4 w-4" />}
          />
        </div>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
          <Card className="col-span-4">
            <CardHeader>
              <CardTitle>Overview</CardTitle>
              <CardDescription>
                Your activity overview for this period.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="h-[200px] flex items-center justify-center border-2 border-dashed border-muted rounded-lg">
                <p className="text-muted-foreground">
                  Chart placeholder - integrate your preferred chart library
                </p>
              </div>
            </CardContent>
          </Card>

          <Card className="col-span-3">
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
              <CardDescription>
                Your latest actions and updates.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <div className="h-2 w-2 rounded-full bg-primary" />
                  <div className="flex-1 space-y-1">
                    <p className="text-sm font-medium leading-none">
                      Account created
                    </p>
                    <p className="text-sm text-muted-foreground">
                      Welcome to {{name}}!
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <div className="h-2 w-2 rounded-full bg-muted" />
                  <div className="flex-1 space-y-1">
                    <p className="text-sm font-medium leading-none">
                      Profile setup
                    </p>
                    <p className="text-sm text-muted-foreground">
                      Complete your profile for a better experience
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </AppLayout>
  )
}
