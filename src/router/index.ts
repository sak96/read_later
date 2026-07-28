import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
const Splash = () => import('../pages/Splash.vue')
const Home = () => import('../pages/Home.vue')
const ArticleDetail = () => import('../pages/ArticleDetail.vue')
const Settings = () => import('../pages/Settings.vue')
const AddArticle = () => import('../pages/AddArticle.vue')
const PronunciationRules = () => import('../pages/PronunciationRules.vue')

const routes = [
  {
    path: '/',
    name: 'splash',
    component: Splash,
  },
  {
    path: '/home',
    name: 'home',
    component: Home,
  },
  {
    path: '/article/:id(\\d+)',
    name: 'article',
    component: ArticleDetail,
    props: (route: RouteLocationNormalized) => ({
      id: Number(route.params.id),
    }),
  },
  {
    path: '/settings',
    name: 'settings',
    component: Settings,
  },
  {
    path: '/add_article/',
    name: 'addArticle',
    component: AddArticle,
  },
  {
    path: '/pronunciation-rules',
    name: 'pronunciationRules',
    component: PronunciationRules,
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
