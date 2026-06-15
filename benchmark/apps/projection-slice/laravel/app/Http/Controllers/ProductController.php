<?php

namespace App\Http\Controllers;

use App\Models\Product;
use Illuminate\Http\Request;

class ProductController extends Controller
{
    // Browse — list page (parity with the projection's Browse/DataTable).
    public function index()
    {
        $products = Product::all();

        return view('products.index', compact('products'));
    }

    // Focus — detail page (parity with the projection's Focus/Card).
    public function show(Product $product)
    {
        return view('products.show', compact('product'));
    }

    // Summarize — stat page (parity with the projection's Summarize/StatCard).
    public function summary()
    {
        $total = Product::sum('price');

        return view('products.summary', compact('total'));
    }

    // Process — kanban grouped by status (parity with the projection's Kanban).
    public function board()
    {
        $columns = [
            'draft' => Product::where('status', 'draft')->get(),
            'active' => Product::where('status', 'active')->get(),
            'discontinued' => Product::where('status', 'discontinued')->get(),
        ];

        return view('products.board', compact('columns'));
    }

    // Collect — create form (parity with the projection's Collect/Form).
    public function create()
    {
        return view('products.create');
    }

    // Collect — store the submitted form (the projection leaves this to the
    // consumer; included here so the create surface is a working round-trip).
    public function store(Request $request)
    {
        $data = $request->validate([
            'name' => 'required|string',
            'price' => 'required|numeric',
            'stock' => 'required|integer',
            'status' => 'required|string',
        ]);

        Product::create($data);

        return redirect()->route('products.index');
    }
}
