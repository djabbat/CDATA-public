//! P61 — Агент-ориентированная модель старения (Уровень +4)
//!
//! Каждый «агент» = один организм (одна ECS-сущность Blood HSC).
//! Агенты взаимодействуют через системный SASP:
//!   сенесцентные клетки одного организма повышают воспаление у «соседей»
//!   (эпидемиология старения, паракринное распространение SASP).

/// Результат симуляции одного агента-организма.
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_id: usize,
    pub lifespan_years: f32,
    pub caii_final: f32,
    pub senescent_fraction_final: f32,
    /// Суммарный SASP, полученный от соседей за всё время жизни.
    pub received_sasp_burden: f32,
    pub death_cause: String,
}

/// Параметры популяции агентов.
#[derive(Debug, Clone)]
pub struct AgentPopulationParams {
    /// Число агентов в популяции. По умолчанию: 20.
    pub n_agents: usize,
    /// Доля SASP, передаваемая соседям за один шаг. По умолчанию: 0.05.
    pub sasp_transmission_rate: f32,
    /// Число соседей, на которых влияет SASP каждого агента. По умолчанию: 3.
    pub interaction_radius: usize,
    /// Ускорение времени: шагов/год (365.0 = 1 шаг/день). По умолчанию: 365.0.
    pub time_acceleration: f64,
    /// Максимальный возраст симуляции в годах. По умолчанию: 100.0.
    pub max_age_years: f32,
}

impl Default for AgentPopulationParams {
    fn default() -> Self {
        Self {
            n_agents: 20,
            sasp_transmission_rate: 0.05,
            interaction_radius: 3,
            time_acceleration: 365.0,
            max_age_years: 100.0,
        }
    }
}

/// Сводная статистика по популяции агентов.
#[derive(Debug, Clone)]
pub struct AgentPopulationStats {
    pub mean_lifespan: f32,
    pub sd_lifespan: f32,
    pub mean_received_sasp: f32,
    /// Доля агентов, у которых received_sasp_burden > 0.1.
    pub fraction_sasp_accelerated: f32,
}

impl AgentPopulationStats {
    /// Рассчитать статистику из вектора результатов агентов.
    pub fn from_results(results: &[AgentResult]) -> Self {
        let n = results.len() as f32;
        if n == 0.0 {
            return Self {
                mean_lifespan: 0.0,
                sd_lifespan: 0.0,
                mean_received_sasp: 0.0,
                fraction_sasp_accelerated: 0.0,
            };
        }
        let mean_lifespan = results.iter().map(|r| r.lifespan_years).sum::<f32>() / n;
        let sd_lifespan = {
            let var = results
                .iter()
                .map(|r| (r.lifespan_years - mean_lifespan).powi(2))
                .sum::<f32>()
                / n;
            var.sqrt()
        };
        let mean_received_sasp =
            results.iter().map(|r| r.received_sasp_burden).sum::<f32>() / n;
        let fraction_sasp_accelerated = results
            .iter()
            .filter(|r| r.received_sasp_burden > 0.1)
            .count() as f32
            / n;
        Self {
            mean_lifespan,
            sd_lifespan,
            mean_received_sasp,
            fraction_sasp_accelerated,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Упрощённый внутренний симулятор одного агента (Blood HSC)
// ─────────────────────────────────────────────────────────────────────────────

/// Внутреннее состояние одного агента для симуляции.
#[derive(Debug, Clone)]
struct AgentState {
    age_years: f32,
    ros_level: f32,
    caii: f32,
    senescent_fraction: f32,
    sasp_output: f32,
    frailty: f32,
    received_sasp: f32,
}

impl AgentState {
    fn new_young() -> Self {
        Self {
            age_years: 0.0,
            ros_level: 0.05,
            caii: 0.95,
            senescent_fraction: 0.01,
            sasp_output: 0.005,
            frailty: 0.0,
            received_sasp: 0.0,
        }
    }

    /// Один шаг симуляции (dt в годах), с учётом внешнего SASP-буста.
    fn step(&mut self, dt: f32, external_sasp_boost: f32) {
        self.age_years += dt;

        // Внешний SASP добавляется к received_sasp (накопительно)
        self.received_sasp += external_sasp_boost * dt;

        // ROS нарастает с возрастом + внешний SASP-буст
        let age_ros_rate = 0.003;
        let ros_boost_from_sasp = external_sasp_boost * 0.8;
        self.ros_level = (self.ros_level + (age_ros_rate + ros_boost_from_sasp) * dt)
            .clamp(0.0, 1.0);

        // CAII снижается с возрастом и ROS
        let caii_decline = (0.005 + self.ros_level * 0.02) * dt;
        self.caii = (self.caii - caii_decline).clamp(0.0, 1.0);

        // Сенесцентная фракция нарастает от ROS и внешнего SASP
        let senes_rate = (self.ros_level * 0.04 + self.sasp_output * 0.06
            + external_sasp_boost * 0.10)
            * dt;
        self.senescent_fraction = (self.senescent_fraction + senes_rate).clamp(0.0, 1.0);

        // SASP пропорционален сенесцентной фракции
        self.sasp_output = (self.senescent_fraction * 0.60).clamp(0.0, 1.0);

        // Frailty нарастает от потери CAII и сенесценции
        let frailty_rate = ((1.0 - self.caii) * 0.06 + self.senescent_fraction * 0.04) * dt;
        self.frailty = (self.frailty + frailty_rate).clamp(0.0, 1.0);
    }

    fn is_dead(&self, max_age: f32) -> Option<String> {
        if self.frailty >= 0.95 {
            return Some("frailty".to_string());
        }
        if self.caii <= 0.05 {
            return Some("stem_exhaustion".to_string());
        }
        if self.age_years >= max_age {
            return Some("max_age".to_string());
        }
        None
    }
}

/// Симулировать одного агента до смерти.
///
/// `ros_boost_initial` — дополнительный стартовый ROS от SASP соседей
/// (применяется как постоянный boost поверх естественного nарастания).
pub fn simulate_single_agent(
    agent_id: usize,
    params: &AgentPopulationParams,
    received_sasp_burden: f32,
) -> AgentResult {
    let dt = 1.0 / params.time_acceleration as f32;
    let max_steps = (params.max_age_years * params.time_acceleration as f32) as usize;

    let mut state = AgentState::new_young();
    // Вносим SASP-нагрузку от соседей как постоянный ros-буст
    let external_sasp_per_step = received_sasp_burden;

    for _ in 0..max_steps {
        state.step(dt, external_sasp_per_step);
        if let Some(cause) = state.is_dead(params.max_age_years) {
            return AgentResult {
                agent_id,
                lifespan_years: state.age_years,
                caii_final: state.caii,
                senescent_fraction_final: state.senescent_fraction,
                received_sasp_burden: state.received_sasp,
                death_cause: cause,
            };
        }
    }

    AgentResult {
        agent_id,
        lifespan_years: state.age_years,
        caii_final: state.caii,
        senescent_fraction_final: state.senescent_fraction,
        received_sasp_burden: state.received_sasp,
        death_cause: "survived".to_string(),
    }
}

/// Симулировать всю популяцию агентов с SASP-взаимодействием.
///
/// Упрощённая последовательная модель:
/// - Каждый агент симулируется независимо, но получает ros_boost
///   от среднего SASP предыдущих агентов × sasp_transmission_rate.
pub fn simulate_agent_population(params: &AgentPopulationParams) -> Vec<AgentResult> {
    let mut results: Vec<AgentResult> = Vec::with_capacity(params.n_agents);
    let mut cumulative_sasp: f32 = 0.0;

    for agent_id in 0..params.n_agents {
        // Системный SASP = среднее по уже завершённым агентам × transmission_rate
        let mean_sasp = if agent_id > 0 {
            cumulative_sasp / agent_id as f32
        } else {
            0.0
        };
        let sasp_boost = mean_sasp * params.sasp_transmission_rate;

        let result = simulate_single_agent(agent_id, params, sasp_boost);
        cumulative_sasp += result.senescent_fraction_final * 0.60;
        results.push(result);
    }

    results
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 1. Пустой вектор → все поля = 0.
    #[test]
    fn agent_stats_empty() {
        let stats = AgentPopulationStats::from_results(&[]);
        assert_eq!(stats.mean_lifespan, 0.0);
        assert_eq!(stats.sd_lifespan, 0.0);
        assert_eq!(stats.mean_received_sasp, 0.0);
        assert_eq!(stats.fraction_sasp_accelerated, 0.0);
    }

    /// 2. Один агент — статистика совпадает с его значениями.
    #[test]
    fn agent_stats_single() {
        let results = vec![AgentResult {
            agent_id: 0,
            lifespan_years: 80.0,
            caii_final: 0.4,
            senescent_fraction_final: 0.30,
            received_sasp_burden: 0.05,
            death_cause: "frailty".to_string(),
        }];
        let stats = AgentPopulationStats::from_results(&results);
        assert!((stats.mean_lifespan - 80.0).abs() < 1e-5);
        assert!((stats.sd_lifespan - 0.0).abs() < 1e-5);
        assert!((stats.mean_received_sasp - 0.05).abs() < 1e-5);
        assert_eq!(stats.fraction_sasp_accelerated, 0.0); // 0.05 < 0.1
    }

    /// 3. Несколько агентов — правильное среднее продолжительности жизни.
    #[test]
    fn agent_stats_mean_lifespan() {
        let results = vec![
            AgentResult {
                agent_id: 0,
                lifespan_years: 70.0,
                caii_final: 0.3,
                senescent_fraction_final: 0.4,
                received_sasp_burden: 0.0,
                death_cause: "frailty".to_string(),
            },
            AgentResult {
                agent_id: 1,
                lifespan_years: 80.0,
                caii_final: 0.4,
                senescent_fraction_final: 0.3,
                received_sasp_burden: 0.0,
                death_cause: "frailty".to_string(),
            },
            AgentResult {
                agent_id: 2,
                lifespan_years: 90.0,
                caii_final: 0.5,
                senescent_fraction_final: 0.2,
                received_sasp_burden: 0.0,
                death_cause: "max_age".to_string(),
            },
        ];
        let stats = AgentPopulationStats::from_results(&results);
        assert!(
            (stats.mean_lifespan - 80.0).abs() < 1e-4,
            "Ожидалось среднее 80.0, получено {:.4}",
            stats.mean_lifespan
        );
    }

    /// 4. Правильно считается доля агентов с received_sasp_burden > 0.1.
    #[test]
    fn sasp_accelerated_fraction() {
        let results = vec![
            AgentResult {
                agent_id: 0,
                lifespan_years: 75.0,
                caii_final: 0.3,
                senescent_fraction_final: 0.3,
                received_sasp_burden: 0.05, // < 0.1
                death_cause: "frailty".to_string(),
            },
            AgentResult {
                agent_id: 1,
                lifespan_years: 70.0,
                caii_final: 0.2,
                senescent_fraction_final: 0.4,
                received_sasp_burden: 0.15, // > 0.1
                death_cause: "frailty".to_string(),
            },
            AgentResult {
                agent_id: 2,
                lifespan_years: 65.0,
                caii_final: 0.1,
                senescent_fraction_final: 0.5,
                received_sasp_burden: 0.20, // > 0.1
                death_cause: "frailty".to_string(),
            },
            AgentResult {
                agent_id: 3,
                lifespan_years: 85.0,
                caii_final: 0.5,
                senescent_fraction_final: 0.2,
                received_sasp_burden: 0.02, // < 0.1
                death_cause: "max_age".to_string(),
            },
        ];
        let stats = AgentPopulationStats::from_results(&results);
        // 2 из 4 агентов имеют burden > 0.1 → 0.5
        assert!(
            (stats.fraction_sasp_accelerated - 0.5).abs() < 1e-5,
            "Ожидалось 0.5, получено {:.4}",
            stats.fraction_sasp_accelerated
        );
    }

    /// 5. Популяция с высоким transmission_rate → средний SASP выше,
    ///    статистика рассчитывается корректно.
    #[test]
    fn high_sasp_transmission_stats_correct() {
        // Создаём результаты руками: часть с высоким received_sasp
        let results: Vec<AgentResult> = (0..10)
            .map(|i| AgentResult {
                agent_id: i,
                lifespan_years: 75.0 - i as f32,
                caii_final: 0.3,
                senescent_fraction_final: 0.3,
                received_sasp_burden: 0.05 * i as f32, // нарастающий SASP
                death_cause: "frailty".to_string(),
            })
            .collect();

        let stats = AgentPopulationStats::from_results(&results);
        // Среднее lifespan = 75 - (0+1+...+9)/10 = 75 - 4.5 = 70.5
        assert!(
            (stats.mean_lifespan - 70.5).abs() < 1e-3,
            "mean_lifespan={:.4}",
            stats.mean_lifespan
        );
        // SD должно быть > 0 (разные lifespans)
        assert!(
            stats.sd_lifespan > 0.0,
            "sd_lifespan должен быть > 0: {:.4}",
            stats.sd_lifespan
        );
        // mean_received_sasp = 0.05 * (0+1+...+9)/10 = 0.05 * 4.5 = 0.225
        assert!(
            (stats.mean_received_sasp - 0.225).abs() < 1e-4,
            "mean_received_sasp={:.4}",
            stats.mean_received_sasp
        );
    }

    /// 6. simulate_agent_population возвращает правильное число агентов.
    #[test]
    fn population_count_matches_params() {
        let params = AgentPopulationParams {
            n_agents: 5,
            time_acceleration: 12.0, // ускоренная симуляция для теста
            max_age_years: 100.0,
            ..Default::default()
        };
        let results = simulate_agent_population(&params);
        assert_eq!(results.len(), 5, "Должно быть 5 агентов");
        for (i, r) in results.iter().enumerate() {
            assert_eq!(r.agent_id, i);
            assert!(r.lifespan_years > 0.0);
        }
    }
}
