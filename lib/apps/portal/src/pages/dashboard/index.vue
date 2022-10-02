<script lang="ts" setup>
import { createPocoClient, PocoClient, PocoServiceRole } from 'poco-client';
import { PocoServiceEntry } from 'poco-client/dist/client/type';
import { onMounted, onUnmounted, toRef } from 'vue';
import { useRoute } from 'vue-router';
import { $, $computed, $ref } from 'vue/macros';
import web3 from "web3";
import StatusIndicator from '../../components/StatusIndicator.vue';
import backgroundUrl from "../../assets/background.png";

const route = useRoute();
const role = $(toRef(route.query, "role"))

let client = $ref<PocoClient>();
let account = $ref("")
let balance = $ref("")
let blockNum = $ref("");
let timer = $ref<number | null>(null);
let services = $ref<PocoServiceEntry[]>([])
let jobs = $ref<PocoJob[]>([]);

const getBlockNumber = async () => {
    const num = await client.getBlockNumber();

    blockNum = num.toString();
}

const getBalance = async () => {
    const wei = await client.getBalance();
    const ether = web3.utils.fromWei(wei, "ether");

    balance = ether.substring(0, ether.indexOf(".") + 5);
}

const getServices = async () => {
    services = client.getServices(PocoServiceRole.MESSENGER);
}

const getJobs = async () => {
    jobs = client.getJobs();
}

const postJob = async () => {
    await client.postJob()
}

onMounted(async () => {
    client = await createPocoClient();

    account = client.localAddress;

    await getBlockNumber();
    await getBalance();
    getServices();
    getJobs();

    timer = setInterval(async () => {
        await getBlockNumber();
        await getBalance();
        getServices();
        getJobs();
    }, 5000) as unknown as number;
})

onUnmounted(() => {
    if (timer) {
        clearInterval(timer)
    }
})
</script>

<template>
    <div class="min-h-screen flex flex-col relative">
        <div class="absolute inset-0 -z-10">
            <img :src="backgroundUrl" class="w-full h-full object-cover" />
        </div>
        <div class="p-8 flex-grow flex flex-col">
            <div class="grid grid-cols-12 grid-rows-3 gap-6 flex-grow">
                <!-- info -->
                <div
                    class="bg-white/20 backdrop-blur-md col-span-12 row-start-1 col-start-1 md:col-span-12 lg:col-span-5 row-span-1 p-4 border border-gray-200 rounded-md shadow-md overflow-hidden">
                    <h2 class="text-xl uppercase text-white/80">info</h2>
                    <div class="py-2">
                        <table class="w-full text-sm text-left text-white rounded-md overflow-hidden table-fixed">
                            <thead class="text-xs text-white uppercase border-b-2">
                                <th scope="col" class="w-48 py-3 px-6">
                                    Property
                                </th>
                                <th scope="col" class="py-3 px-6">
                                    Value
                                </th>
                            </thead>
                            <tbody>
                                <tr class="border-b">
                                    <th scope="row"
                                        class="py-2 px-6 font-medium text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                        Account
                                    </th>
                                    <td class="py-3 px-6 text-ellipsis overflow-hidden">{{account}}</td>
                                </tr>
                                <tr class="border-b">
                                    <th scope="row"
                                        class="py-2 px-6 font-medium text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                        Balance
                                    </th>
                                    <td class="py-3 px-6 text-ellipsis overflow-hidden">{{balance}}</td>
                                </tr>
                                <tr class="border-b">
                                    <th scope="row"
                                        class="py-2 px-6 font-medium text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                        BlockNum
                                    </th>
                                    <td class="py-3 px-6 text-ellipsis overflow-hidden">{{blockNum}}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
                <!-- messenger -->
                <div
                    class="bg-white/20 backdrop-blur-md col-span-full md:col-span-full lg:col-span-5 lg:row-span-2 row-span-1 row-start-2 p-4 border border-gray-200 rounded-md shadow-md overflow-hidden">
                    <h2 class="text-xl uppercase text-white/80 mb-2">messenger</h2>
                    <table
                        class="w-full text-sm text-left text-gray-500 dark:text-gray-400 rounded-md overflow-hidden table-fixed">
                        <thead class="text-xs text-white uppercase border-b-2">
                            <tr class="table-fixed">
                                <th scope="col" class="py-3 px-6">
                                    User
                                </th>
                                <th scope="col" class="py-3 px-6">
                                    Endpoint
                                </th>
                                <th scope="col" class="py-3 px-6">
                                    Status
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr class="border-b table-fixed" v-for="item in services" :key="item.user">
                                <th scope="row"
                                    class="py-2 px-6 font-medium text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                    {{item.user}}
                                </th>
                                <td class="py-4 px-6 text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                    {{item.endpoint}}
                                </td>
                                <td class="py-4 px-6 uppercase inline-flex items-baseline gap-1 text-white">
                                    <StatusIndicator :status="item.status" size="default"></StatusIndicator>
                                    <span class="uppercase">{{item.status}}</span>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                <!-- jobs -->
                <div
                    class="bg-white/20 backdrop-blur-md col-span-full row-span-1 row-start-3 lg:col-span-7 lg:row-span-3 lg:row-start-1 lg:col-start-6 p-4 border border-gray-200 rounded-md shadow-md overflow-hidden">
                    <div class="flex justify-between">
                        <h2 class="text-xl uppercase text-white/80">jobs</h2>
                        <div>
                            <button @click="postJob" type="button"
                                class="text-white bg-gradient-to-br from-purple-600 to-blue-500 hover:-hue-rotate-30 transition-all font-medium rounded-full text-sm p-2.5 text-center inline-flex items-center mr-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                                    xmlns="http://www.w3.org/2000/svg">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12">
                                    </path>
                                </svg>
                            </button>
                        </div>
                    </div>
                    <table
                        class="w-full text-sm text-left text-gray-500 dark:text-gray-400 rounded-md overflow-hidden table-fixed">
                        <thead class="text-xs text-white uppercase border-b-2">
                            <tr class="table-fixed">
                                <th scope="col" class="py-3 px-4 w-12">
                                    JobId
                                </th>
                                <th scope="col" class="py-3 px-6">
                                    From
                                </th>
                                <th scope="col" class="py-3 px-6">
                                    Action & Progress
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr class="border-b table-fixed" v-for="item in jobs" :key="item.jobId">
                                <th scope="row"
                                    class="py-4 px-4 w-12 font-medium text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                    {{item.jobId}}
                                </th>
                                <td class="py-4 px-6 text-white whitespace-nowrap text-ellipsis overflow-hidden">
                                    {{item.owner}}
                                </td>
                                <td class="py-4 px-6 uppercase inline-flex items-baseline gap-1 text-white">
                                    <div class="flex items-center">
                                        <button
                                            :disabled="account.toUpperCase() === item.owner.toUpperCase() || item.status !== 'acceptable'"
                                            type="button"
                                            class="text-white bg-gradient-to-r from-green-400 via-green-500 to-green-600 hover:bg-gradient-to-br font-medium rounded-lg text-sm px-5 py-2.5 text-center disabled:bg-gray-400 disabled:bg-none">
                                            Take
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
</template>