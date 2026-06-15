<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('world', function (Blueprint $table) {
            $table->integer('id')->autoIncrement()->primary();
            $table->integer('randomNumber');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('world');
    }
};
