@extends('layouts.app')

@section('content')
    <div class="stat-card">
        <span class="stat-label">Total Price</span>
        <span class="stat-value">{{ $total }}</span>
    </div>
@endsection
