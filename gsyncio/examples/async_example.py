#!/usr/bin/env python3
"""
Async/Await Example - I/O-bound operations

This example demonstrates the async/await model for I/O-bound
operations with gsyncio's fiber-based scheduler.
"""

import gsyncio as gs


async def fetch_url(url, delay_ms=100):
    """Simulate fetching a URL with delay"""
    await gs.sleep(delay_ms)
    return f"Data from {url}"


async def process_data(data, delay_ms=50):
    """Simulate processing data with delay"""
    await gs.sleep(delay_ms)
    return f"Processed: {data}"


async def main():
    """Main function demonstrating async/await"""
    print("Starting async operations...")
    
    # Create URLs to fetch
    urls = [f"http://example.com/{i}" for i in range(20)]
    
    # Fetch all URLs concurrently
    print(f"Fetching {len(urls)} URLs...")
    fetch_tasks = [gs.create_task(fetch_url(url)) for url in urls]
    results = await gs.gather(*fetch_tasks)
    print(f"Fetched {len(results)} URLs")
    
    # Process results concurrently
    print("Processing results...")
    process_tasks = [gs.create_task(process_data(r)) for r in results]
    processed = await gs.gather(*process_tasks)
    print(f"Processed {len(processed)} items")
    
    # Show first few results
    print("\nFirst 5 results:")
    for r in processed[:5]:
        print(f"  {r}")
    
    # Show scheduler stats
    stats = gs.get_scheduler_stats()
    print(f"\nScheduler stats: {stats}")


if __name__ == '__main__':
    from gsyncio.async_ import run as async_run
    async_run(main())
