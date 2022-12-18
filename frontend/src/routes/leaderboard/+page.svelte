<script lang="ts">
    import { API_URL } from '../../api';
    import { onMount } from 'svelte';
    import Spinner from '../../components/spinner.svelte';
    import QuoteComponent from '../../components/quote.svelte';
    import type { Quote } from '../../models';
    import { fade } from 'svelte/transition';
    import { Bar } from 'svelte-chartjs';
    import {
        Chart,
        Title,
        Tooltip,
        Legend,
        BarElement,
        CategoryScale,
        LinearScale
    } from 'chart.js';

    Chart.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale);
    Chart.defaults.color = '#CBD5E1'; // slate-300
    Chart.defaults.borderColor = '#334155'; // slate-300

    let leaderboardLoading = true;
    let leaderboard: Quote[] | null = null;
    let leaderboardError = false;

    let funniestPeopleLoading = true;
    let funniestPeople = null;
    let funniestPeopleError = false;

    let allScoresLoading = true;
    let allScores: number[] | null = null;
    let allScoresError = false;

    let top100ChartData = {};

    let distrubitonChartData = {};

    onMount(() => {
        fetchLeaderboard();
        fetchFunniestPeople();
        fetchAllScores();
    });

    async function fetchLeaderboard() {
        leaderboardLoading = true;
        try {
            let res = await fetch(API_URL + '/leaderboard');
            leaderboard = await res.json();
            leaderboardError = false;

            const usersSummedScores = leaderboard?.reduce(
                (prev: any, quote: Quote) => ({
                    ...prev,
                    [quote.author_id]: (prev[quote.author_id] || 0) + quote.score
                }),
                {}
            );

            const usernames = leaderboard?.reduce(
                (prev: any, quote: Quote) => ({
                    ...prev,
                    [quote.author_id]: quote.username
                }),
                {}
            );

            const usersQuoteCount = Object.entries(
                leaderboard?.reduce(
                    (prev: any, quote: Quote) => ({
                        ...prev,
                        [quote.author_id]: (prev[quote.author_id] || 0) + 1
                    }),
                    {}
                )
            )
                .sort((a: [string, any], b: [string, any]) => b[1] - a[1])
                .map((x) => [...x, usernames[x[0]]])
                .filter((x) => x[2]); // remove empty usernames

            top100ChartData = {
                labels: usersQuoteCount.map((x) => x[2]),
                datasets: [
                    {
                        label: 'Amount of quotes in top 100',
                        data: usersQuoteCount.map((x) => x[1]),
                        backgroundColor: ['rgba(244, 63, 94, 0.4)'],
                        borderColor: ['rgba(244, 63, 94, 0.8)'],
                        borderWidth: 1
                    }
                ]
            };
            console.log(top100ChartData);
        } catch {
            leaderboardError = true;
        }
        leaderboardLoading = false;
    }

    async function fetchFunniestPeople() {
        funniestPeopleLoading = true;

        try {
            let res = await fetch(API_URL + '/funniest-people');
            funniestPeople = await res.json();
            funniestPeopleError = false;
        } catch {
            funniestPeopleError = true;
        }

        funniestPeopleLoading = false;
    }

    async function fetchAllScores() {
        allScoresLoading = true;

        console.log(histogram(Array(100).fill(Math.random()), 10));

        try {
            const res = await fetch(API_URL + '/all-scores');
            allScores = await res.json();
            allScoresError = false;

            const labels = Array(10)
                .fill(null)
                .map((_, i) => `${i * 10}%`);
            console.log(labels);

            top100ChartData = {
                labels: labels,
                datasets: [
                    {
                        label: 'Amount of quotes in top 100',
                        data: histogram(
                            (allScores || []).map((x) => x * 100),
                            10
                        ),
                        backgroundColor: ['rgba(244, 63, 94, 0.4)'],
                        borderColor: ['rgba(244, 63, 94, 0.8)'],
                        borderWidth: 1
                    }
                ]
            };
            console.log(top100ChartData);
        } catch {
            allScoresError = true;
        }

        allScoresLoading = false;
    }

    async function onVote(id: number, vote: number) {
        const res = await fetch(API_URL + `/vote/${id}/${vote}`, { method: 'POST' });
        fetchLeaderboard();
    }
</script>

<main
    class="w-screen min-h-screen bg-slate-900 text-slate-300 flex justify-center overflow-x-hidden"
>
    <div class="sm:mt-12 w-[min(900px,90%)]">
        <a
            href="/"
            aria-label="Go back"
            class="inline-block justify-self-start ml-1 mr-auto justify-center rounded-full p-2 text-sm font-medium text-slate-300  focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-100 transition-all"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
                class="w-6 h-6"
                aria-hidden="true"
            >
                <path
                    fill-rule="evenodd"
                    d="M11.03 3.97a.75.75 0 010 1.06l-6.22 6.22H21a.75.75 0 010 1.5H4.81l6.22 6.22a.75.75 0 11-1.06 1.06l-7.5-7.5a.75.75 0 010-1.06l7.5-7.5a.75.75 0 011.06 0z"
                    clip-rule="evenodd"
                />
            </svg>
        </a>
        <div class=" bg-slate-800 rounded-xl p-6 shadow-lg mb-4">
            <h1 class="text-2xl">Top 100</h1>
            <p>Pls don't spam we gotta appreciate the tastiest fruits 🍉</p>

            {#if leaderboardLoading}
                <div
                    class="flex justify-center items-center h-72"
                    in:fade={{ duration: 150 }}
                    out:fade={{ duration: 150 }}
                >
                    <Spinner />
                </div>
            {:else}
                <div class="flex flex-col gap-4 mt-4" in:fade={{ duration: 150 }}>
                    <Bar
                        data={top100ChartData}
                        responsive={true}
                        height={200}
                        options={{
                            scales: {
                                x: { grid: { display: false } },
                                y: { grid: { display: false } }
                            },
                            elements: { bar: { borderRadius: 4 } },
                            plugins: { legend: { onClick: () => {} } }
                        }}
                    />
                    {#each leaderboard || [] as quote}
                        <QuoteComponent {quote} compact={true} {onVote} />
                    {/each}
                    <Bar
                        data={distrubitonChartData}
                        responsive={true}
                        height={200}
                        options={{
                            scales: {
                                x: { grid: { display: false } },
                                y: { grid: { display: false } }
                            },
                            elements: { bar: { borderRadius: 4 } },
                            plugins: { legend: { onClick: () => {} } }
                        }}
                    />
                </div>
            {/if}
        </div>
    </div>
</main>
