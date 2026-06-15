@extends('layouts.app')

@section('content')
    <h1>Product Board</h1>

    <div class="board">
        @foreach ($columns as $status => $products)
            <div class="column">
                <h2>{{ ucfirst($status) }} ({{ $products->count() }})</h2>
                @foreach ($products as $product)
                    <div class="card">
                        <strong>{{ $product->name }}</strong>
                        <span>{{ $product->price }}</span>
                    </div>
                @endforeach
            </div>
        @endforeach
    </div>
@endsection
