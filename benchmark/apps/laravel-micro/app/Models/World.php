<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class World extends Model
{
    protected $table = 'world';
    public $timestamps = false;
    // DB column is random_number (snake_case, created by the shared ferro migration).
    protected $fillable = ['random_number'];
}
