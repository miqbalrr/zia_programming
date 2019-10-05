/*  Library for the Zia programming language.
    Copyright (C) 2018 to 2019 Charles Johnson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

use delta::ApplyDelta;
use errors::ZiaResult;

pub trait DefinitionDeleter: ApplyDelta {
    fn cleanly_delete_definition(&self, delta: &mut Self::Delta, concept: usize) -> ZiaResult<()>;
    fn try_delete_concept(
        &self,
        previous_deltas: &mut Self::Delta,
        concept: usize,
    ) -> ZiaResult<()>;
}

pub trait BlindConceptRemover {
    fn blindly_remove_concept(&mut self, usize);
}

pub trait BlindConceptRemoverDelta
where
    Self: ApplyDelta,
{
    fn blindly_remove_concept_delta(&self, &mut Self::Delta, usize);
}

pub trait StringRemover {
    fn remove_string(&mut self, &str);
}

pub trait StringRemoverDelta
where
    Self: ApplyDelta,
{
    fn remove_string_delta(&self, &mut Self::Delta, &str);
}
