//! Explicit mission lifecycle with evidence-gated completion.
//! Permissions, review authenticity, and durable persistence are future work.
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Open,
    Claimed { actor: String },
    Submitted { actor: String, evidence_id: String },
    Completed { actor: String, evidence_id: String },
}

#[derive(Clone, Debug)]
pub struct Mission {
    pub id: String,
    pub title: String,
    pub status: Status,
}

#[derive(Default)]
pub struct MissionBoard {
    missions: BTreeMap<String, Mission>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MissionError {
    InvalidInput,
    DuplicateMission,
    NotFound,
    InvalidTransition,
}

impl MissionBoard {
    pub fn create(&mut self, id: &str, title: &str) -> Result<(), MissionError> {
        if id.trim().is_empty() || title.trim().is_empty() {
            return Err(MissionError::InvalidInput);
        }
        if self.missions.contains_key(id) {
            return Err(MissionError::DuplicateMission);
        }
        self.missions.insert(id.into(), Mission {
            id: id.into(),
            title: title.into(),
            status: Status::Open,
        });
        Ok(())
    }

    pub fn claim(&mut self, id: &str, actor: &str) -> Result<(), MissionError> {
        if actor.trim().is_empty() {
            return Err(MissionError::InvalidInput);
        }
        let mission = self.missions.get_mut(id).ok_or(MissionError::NotFound)?;
        if mission.status != Status::Open {
            return Err(MissionError::InvalidTransition);
        }
        mission.status = Status::Claimed { actor: actor.into() };
        Ok(())
    }

    pub fn submit(&mut self, id: &str, actor: &str, evidence_id: &str) -> Result<(), MissionError> {
        if evidence_id.trim().is_empty() {
            return Err(MissionError::InvalidInput);
        }
        let mission = self.missions.get_mut(id).ok_or(MissionError::NotFound)?;
        match &mission.status {
            Status::Claimed { actor: owner } if owner == actor => {
                mission.status = Status::Submitted {
                    actor: actor.into(),
                    evidence_id: evidence_id.into(),
                };
                Ok(())
            }
            _ => Err(MissionError::InvalidTransition),
        }
    }

    pub fn approve(&mut self, id: &str) -> Result<(), MissionError> {
        let mission = self.missions.get_mut(id).ok_or(MissionError::NotFound)?;
        match &mission.status {
            Status::Submitted { actor, evidence_id } => {
                mission.status = Status::Completed {
                    actor: actor.clone(),
                    evidence_id: evidence_id.clone(),
                };
                Ok(())
            }
            _ => Err(MissionError::InvalidTransition),
        }
    }

    pub fn get(&self, id: &str) -> Option<&Mission> {
        self.missions.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_lifecycle() {
        let mut board = MissionBoard::default();
        board.create("m1", "Document a hive").unwrap();
        board.claim("m1", "agent-a").unwrap();
        board.submit("m1", "agent-a", "evidence-1").unwrap();
        board.approve("m1").unwrap();
        assert!(matches!(board.get("m1").unwrap().status, Status::Completed { .. }));
    }

    #[test]
    fn blocks_unclaimed_submission() {
        let mut board = MissionBoard::default();
        board.create("m1", "Document a hive").unwrap();
        assert_eq!(
            board.submit("m1", "agent-a", "evidence-1"),
            Err(MissionError::InvalidTransition)
        );
    }

    #[test]
    fn blocks_other_actor() {
        let mut board = MissionBoard::default();
        board.create("m1", "Document a hive").unwrap();
        board.claim("m1", "agent-a").unwrap();
        assert_eq!(
            board.submit("m1", "agent-b", "evidence-1"),
            Err(MissionError::InvalidTransition)
        );
    }
}
