@extends('layouts.app')

@section('content')
    <h1>New Product</h1>

    <form method="POST" action="{{ route('products.store') }}">
        @csrf

        <label>Name
            <input type="text" name="name" value="{{ old('name') }}" required>
        </label>

        <label>Price
            <input type="number" step="0.01" name="price" value="{{ old('price') }}" required>
        </label>

        <label>Stock
            <input type="number" name="stock" value="{{ old('stock', 1) }}" required>
        </label>

        <label>Status
            <select name="status" required>
                <option value="draft" @selected(old('status') === 'draft')>Draft</option>
                <option value="active" @selected(old('status') === 'active')>Active</option>
                <option value="discontinued" @selected(old('status') === 'discontinued')>Discontinued</option>
            </select>
        </label>

        <button type="submit">Create</button>
    </form>
@endsection
