// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Piece } from "./Piece";
import type { PieceColor } from "./PieceColor";

export interface Board { pieces: Array<Piece>, selected_piece: Piece | null, en_passant: Piece | null, white_castle_kingside: boolean, white_castle_queenside: boolean, black_castle_kingside: boolean, black_castle_queenside: boolean, turn: PieceColor, ai: PieceColor, max_pid: number, }