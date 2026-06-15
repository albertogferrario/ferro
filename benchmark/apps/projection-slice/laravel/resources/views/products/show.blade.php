@extends('layouts.app')

@section('content')
    <h1>{{ $product->name }}</h1>

    <dl>
        <dt>Name</dt>
        <dd>{{ $product->name }}</dd>

        <dt>Price</dt>
        <dd>{{ $product->price }}</dd>

        <dt>Stock</dt>
        <dd>{{ $product->stock }}</dd>

        <dt>Status</dt>
        <dd>{{ $product->status }}</dd>
    </dl>

    <a href="{{ route('products.index') }}">Back</a>
@endsection
