<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class Product extends Model
{
    protected $fillable = [
        'name',
        'price',
        'stock',
        'status',
    ];

    protected $casts = [
        'price' => 'float',
        'stock' => 'integer',
    ];
}
