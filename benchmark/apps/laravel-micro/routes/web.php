<?php

// benchmark/apps/laravel-micro/routes/web.php
use Illuminate\Support\Facades\Route;
use App\Models\World;

Route::get('/json', fn () => response()->json(['message' => 'Hello, World!']));

Route::get('/db', function () {
    $w = World::find(random_int(1, 10000));
    // DB column is random_number (snake_case); JSON contract key is randomNumber.
    return response()->json(['id' => $w->id, 'randomNumber' => $w->random_number]);
});

$clamp = fn ($n) => max(1, min(500, (int) ($n ?: 1)));

Route::get('/queries', function (\Illuminate\Http\Request $r) use ($clamp) {
    $k = $clamp($r->query('n'));
    $out = [];
    for ($i = 0; $i < $k; $i++) {
        $w = World::find(random_int(1, 10000));
        $out[] = ['id' => $w->id, 'randomNumber' => $w->random_number];
    }
    return response()->json($out);
});

Route::get('/updates', function (\Illuminate\Http\Request $r) use ($clamp) {
    $k = $clamp($r->query('n'));
    $out = [];
    for ($i = 0; $i < $k; $i++) {
        $w = World::find(random_int(1, 10000));
        $w->random_number = random_int(1, 10000);
        $w->save();
        $out[] = ['id' => $w->id, 'randomNumber' => $w->random_number];
    }
    return response()->json($out);
});
