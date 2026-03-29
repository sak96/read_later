import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
import Home from '../pages/Home.vue'
import ArticleDetail from '../pages/ArticleDetail.vue'
import Settings from '../pages/Settings.vue'
import AddArticle from '../pages/AddArticle.vue'

const routes = [
  {
    path: '/',
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
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
