// mutex = protect
// semaphore = count
// condition_variable = sleep until a condition is true
#include <chrono>
#include <condition_variable>
#include <iostream>
#include <mutex>
#include <optional>
#include <queue>
#include <semaphore>
#include <thread>

using namespace std::chrono_literals;

// ============================================================
// 1. MUTEX
//
// Purpose: protect shared data.
//
// It does NOT wait for a particular condition.
// It only allows one thread at a time into a critical section.
// ============================================================

void mutex_example() {
    std::cout << "\n=== 1. Mutex: protect shared data ===\n";

    std::mutex mutex;
    int counter = 0;

    auto increment = [&] {
        for (int i = 0; i < 100'000; ++i) {
            std::lock_guard lock{mutex};

            // Only one thread at a time can modify counter.
            ++counter;
        }
    };

    std::jthread first{increment};
    std::jthread second{increment};

    first.join();
    second.join();

    std::cout << "Counter: " << counter << '\n';
}

// ============================================================
// 2. SEMAPHORE
//
// Purpose: count available permits/events.
//
// release() adds a permit.
// acquire() consumes a permit.
//
// A semaphore remembers permits even if release() happens
// before acquire().
// ============================================================

void semaphore_example() {
    std::cout << "\n=== 2. Semaphore: count permits ===\n";

    // Maximum count: 10
    // Initial count: 0
    std::counting_semaphore<10> permits{0};

    std::jthread producer{[&] {
        std::cout << "Producer: creating 3 permits\n";

        permits.release();
        permits.release();
        permits.release();

        // The semaphore now remembers that 3 permits exist.
    }};

    producer.join();

    // The producer has already finished, but the three permits
    // were remembered. These calls do not block.
    for (int i = 1; i <= 3; ++i) {
        permits.acquire();
        std::cout << "Main: consumed permit " << i << '\n';
    }
}

// ============================================================
// 3. CONDITION VARIABLE
//
// Purpose: sleep until shared state satisfies a condition.
//
// The condition variable does NOT store the condition.
// The actual condition is stored in shared variables.
//
// notify_one() only says:
//     "Wake up and check the shared state again."
// ============================================================

void condition_variable_example() {
    std::cout << "\n=== 3. Condition variable: wait for a condition ===\n";

    std::mutex mutex;
    std::condition_variable cv;

    bool data_ready = false;
    bool shutdown_requested = false;
    int data = 0;

    std::jthread worker{[&] {
        std::unique_lock lock{mutex};

        std::cout << "Worker: waiting for data OR shutdown\n";

        // 1. Worker unlocks mutex
        // 2. Worker goes to sleep
        cv.wait(lock, [&] {
            // This is the condition/predicate.
            return data_ready || shutdown_requested;
        });

        // wait() locks the mutex again before returning.
        if (data_ready) {
            std::cout << "Worker: received data = " << data << '\n';
        } else {
            std::cout << "Worker: shutting down\n";
        }
    }};

    std::this_thread::sleep_for(500ms);

    {
        std::lock_guard lock{mutex};

        data = 42;
        data_ready = true;
    } // Unlock mutex.

    // Wake one waiting thread so it can check the condition.
    cv.notify_one();

    worker.join();
}

void conditional_variable_practice() {
    std::cout << "\n=== 4. Condition variable practice ===\n";
    std::mutex mutex;
    std::condition_variable cv;

    bool ready = false;
    int value = 0;

    std::jthread receiver{[&] {
        std::unique_lock lock(mutex); // lock the mutex

        std::println("Receiver waiting");
        // release the mutex, let producer continue the work
        // and wait until ready is true
        // Make sure the lambda is capturing reference
        cv.wait(lock, [&] { return ready; });
        // If ready is true and this block end, the mutex got locked again

        // Now ready is true, and mutex is locked by this thread
        // perform
        value *= 2;
        std::println("Value after receiver done working {}", value);
    }};

    // producer thread
    std::jthread producer{[&] {
        std::unique_lock lock(mutex);

        value += 1;
        std::println("Value after producer done working {}", value);

        ready = true;

        lock.unlock(); // (optional)
        cv.notify_one();
    }};
}

void semaphore_practice() {
    std::cout << "\n=== 5. Semaphore practice ===\n";
    std::counting_semaphore<100> sem{0};
    std::mutex mutex;
    std::queue<int> jobs;

    constexpr int consumer_count = 4;

    std::jthread producer{[&] {
        // 12 jobs
        for (int job = 1; job <= 12; job++) {
            {
                // lock the queue then push a job
                std::lock_guard lock{mutex};
                jobs.push(job);
            }
            sem.release(); // increase the sem by 1
        }

        // NOTE: Increase by consumer counts at the end so all consumers cana
        // acquire the sem and check for empty then exit
        sem.release(consumer_count);
    }};

    std::vector<std::thread> consumers;

    // spawning 4 consumer
    for (int i = 0; i < consumer_count; i++) {

        std::thread consumer{[&] {
            std::thread::id thread_id = std::this_thread::get_id();

            while (true) {
                sem.acquire(); // decrease the sem by 1

                int job;
                {
                    std::lock_guard lock{mutex};
                    if (jobs.empty()) {
                        std::println("Consumer {} is done. No more job",
                                     thread_id);
                        break;
                    }
                    job = jobs.front(); // get the first job
                    jobs.pop();         // remove the job from the queue
                }

                // Access optional type with *
                std::println("Consumer {} is working on job {}", thread_id,
                             job);
                std::this_thread::sleep_for(200ms); // simuate some work
            }
        }};

        consumers.push_back(std::move(consumer));
    }

    // join all consumers
    for (auto &consumer : consumers) {
        consumer.join();
    }
    std::println("All consumers are done");
}

int main() {
    mutex_example();
    semaphore_example();
    condition_variable_example();
    conditional_variable_practice();
    semaphore_practice();
}
