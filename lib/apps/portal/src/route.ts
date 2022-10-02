import PortalPage from "./pages/portal/index.vue";
import DashboardPage from "./pages/dashboard/index.vue";

export const routes = [
    { name: "portal", path: "/", component: PortalPage },
    { name: "dashboard", path: "/dashboard", component: DashboardPage },
]