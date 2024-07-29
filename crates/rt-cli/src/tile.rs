use rayon::iter::plumbing::bridge;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub x_start: u32,
    pub x_end: u32,
    pub y_start: u32,
    pub y_end: u32,
}

impl Tile {
    pub fn width(&self) -> usize {
        (self.x_end - self.x_start) as usize
    }
    pub fn height(&self) -> usize {
        (self.y_end - self.y_start) as usize
    }
    pub fn len(&self) -> usize {
        self.width() * self.height()
    }
}

pub struct TileIter {
    tile: Tile,
    index: u32,
}

impl Iterator for TileIter {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tile.len() as u32 {
            return None;
        }
        let x = self.index % self.tile.width() as u32;
        let y = self.index / self.tile.width() as u32;
        self.index += 1;
        Some((self.tile.x_start + x, self.tile.y_start + y))
    }
}

impl IntoIterator for Tile {
    type Item = (u32, u32);

    type IntoIter = TileIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            tile: self,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tiler {
    pub offset_x: u32,
    pub offset_y: u32,
    pub width: u32,
    pub height: u32,
    pub x_grainsize: u32,
    pub y_grainsize: u32,
}

fn div_ceil(x: u32, y: u32) -> u32 {
    x / y + if x % y != 0 { 1 } else { 0 }
}

impl Tiler {
    pub fn tile_dimensions(&self) -> (usize, usize) {
        (
            div_ceil(self.width, self.x_grainsize) as usize,
            div_ceil(self.height, self.y_grainsize) as usize,
        )
    }
    pub fn tile_count(&self) -> usize {
        let (r, c) = self.tile_dimensions();
        r * c
    }

    pub fn tile(&self, idx: usize) -> Option<Tile> {
        if idx >= self.tile_count() {
            return None;
        }

        let (col_count, _) = self.tile_dimensions();

        let x = idx as u32 % col_count as u32;
        let y = idx as u32 / col_count as u32;

        Some(Tile {
            x_start: self.offset_x + x * self.x_grainsize,
            x_end: self.offset_x + u32::min(self.width, (x + 1) * self.x_grainsize),
            y_start: self.offset_y + y * self.y_grainsize,
            y_end: self.offset_y + u32::min(self.height, (y + 1) * self.y_grainsize),
        })
    }
}
impl IntoIterator for Tiler {
    type Item = Tile;

    type IntoIter = TileIterator;

    fn into_iter(self) -> Self::IntoIter {
        TileIterator {
            tiler: self,
            start: 0,
            end: self.tile_count(),
        }
    }
}

pub struct TileIterator {
    tiler: Tiler,
    start: usize,
    end: usize,
}

impl ExactSizeIterator for TileIterator {}
impl DoubleEndedIterator for TileIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        self.end -= 1;

        self.tiler.tile(self.end)
    }
}
impl Iterator for TileIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let tile = self.tiler.tile(self.start);
        self.start += 1;
        tile
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }
}

pub struct TileParallelIterator {
    base: Tiler,
}

pub struct TileProducer {
    base: TileIterator,
}

impl rayon::iter::IntoParallelIterator for Tiler {
    type Iter = TileParallelIterator;
    type Item = Tile;

    fn into_par_iter(self) -> Self::Iter {
        Self::Iter { base: self }
    }
}

impl rayon::iter::ParallelIterator for TileParallelIterator {
    type Item = Tile;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }
}
impl rayon::iter::IndexedParallelIterator for TileParallelIterator {
    fn len(&self) -> usize {
        self.base.tile_count()
    }

    fn drive<C: rayon::iter::plumbing::Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: rayon::iter::plumbing::ProducerCallback<Self::Item>>(
        self,
        callback: CB,
    ) -> CB::Output {
        callback.callback(TileProducer {
            base: self.base.into_iter(),
        })
    }
}

impl rayon::iter::plumbing::Producer for TileProducer {
    type Item = Tile;
    type IntoIter = <Tiler as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.base
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        (
            Self {
                base: TileIterator {
                    tiler: self.base.tiler,
                    start: self.base.start,
                    end: self.base.start + index,
                },
            },
            Self {
                base: TileIterator {
                    tiler: self.base.tiler,
                    start: self.base.start + index,
                    end: self.base.end,
                },
            },
        )
    }
}
