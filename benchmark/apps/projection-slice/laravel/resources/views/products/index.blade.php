@extends('layouts.app')

@section('content')
    <h1>Products</h1>

    <table>
        <thead>
            <tr>
                <th>Name</th>
                <th>Price</th>
                <th>Stock</th>
                <th>Status</th>
            </tr>
        </thead>
        <tbody>
            @foreach ($products as $product)
                <tr>
                    <td><a href="{{ route('products.show', $product) }}">{{ $product->name }}</a></td>
                    <td>{{ $product->price }}</td>
                    <td>{{ $product->stock }}</td>
                    <td>{{ $product->status }}</td>
                </tr>
            @endforeach
        </tbody>
    </table>

    <a href="{{ route('products.create') }}">New Product</a>
@endsection
