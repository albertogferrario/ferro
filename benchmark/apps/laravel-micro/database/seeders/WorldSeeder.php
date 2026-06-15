<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;

class WorldSeeder extends Seeder
{
    public function run(): void
    {
        $rows = [];
        for ($i = 0; $i < 10000; $i++) {
            $rows[] = ['randomNumber' => random_int(1, 10000)];
        }
        DB::table('world')->insert($rows);
    }
}
