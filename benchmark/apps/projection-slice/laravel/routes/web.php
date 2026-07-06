<?php

use App\Http\Controllers\ProductController;
use Illuminate\Support\Facades\Route;

Route::get('/products', [ProductController::class, 'index'])->name('products.index');
Route::get('/products/create', [ProductController::class, 'create'])->name('products.create');
Route::post('/products', [ProductController::class, 'store'])->name('products.store');
Route::get('/products/summary', [ProductController::class, 'summary'])->name('products.summary');
Route::get('/products/board', [ProductController::class, 'board'])->name('products.board');
Route::get('/products/{product}', [ProductController::class, 'show'])->name('products.show');
