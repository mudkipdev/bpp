/*
 * Copyright (c) 2025-2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 * Based on code by Mojang Studios (2011)
*/

#pragma once

#include "java_random.h"
#include "noise_perlin.h"
#include <vector>

class NoiseOctavesPerlin {
public:
	NoiseOctavesPerlin() {}
	NoiseOctavesPerlin(int32_t octaves);
	NoiseOctavesPerlin(Java::Random& rand, int32_t octaves, int32_t effectiveOctaves = 0);

	// func_647_a
	double GenerateOctaves(Vec2 offset);
	// generateNoiseOctaves
	void GenerateOctaves(std::vector<double>& noiseField, Vec3 coordinate, Int32_3 size, Vec3 p_scale);
	// func_4103_a
	void GenerateOctaves(std::vector<double>& noiseField, Int32_2 offset, Int32_2 size, Vec2 scale,
	                     [[maybe_unused]] double unused);

private:
	int32_t octaves = 0;
	int32_t firstOctave = 0;
	std::vector<NoisePerlin> generator_collection;
};

inline NoiseOctavesPerlin::NoiseOctavesPerlin(int32_t poctaves) : octaves(poctaves) {
	for (size_t i = 0; i < size_t(octaves); ++i)
		generator_collection.push_back(NoisePerlin());
}

/**
 * @brief Construct a new octave set, sampling only the effectiveOctaves highest-amplitude
 *        octaves at generation time (0 samples all). All octaves are still constructed
 *        so the permutation tables and rand state match vanilla exactly.
 */
inline NoiseOctavesPerlin::NoiseOctavesPerlin(Java::Random& rand, int32_t poctaves, int32_t effectiveOctaves)
    : octaves(poctaves) {
	if (effectiveOctaves > 0 && effectiveOctaves < poctaves)
		firstOctave = poctaves - effectiveOctaves;
	for (size_t i = 0; i < size_t(octaves); ++i)
		generator_collection.push_back(NoisePerlin(rand));
}

inline double NoiseOctavesPerlin::GenerateOctaves(Vec2 offset) {
	double value = 0.0;
	double scale = 1.0;
	for (size_t i = 0; i < size_t(octaves); ++i) {
		value += generator_collection[i].GenerateNoise(offset * scale) / scale;
		scale /= 2.0;
	}
	return value;
}

inline void NoiseOctavesPerlin::GenerateOctaves(std::vector<double>& noiseField, Vec3 coordinate, Int32_3 size,
                                                Vec3 p_scale) {
	size_t sampleCount = size_t(size.x * size.y * size.z);
	if (noiseField.size() != sampleCount)
		noiseField.resize(sampleCount, 0.0);

	if (firstOctave >= octaves) {
		for (size_t i = 0; i < noiseField.size(); ++i)
			noiseField[i] = 0.0;
		return;
	}

	double multiplier = 1.0;
	for (int32_t octave = 0; octave < firstOctave; ++octave)
		multiplier /= 2.0;

	for (int32_t octave = firstOctave; octave < octaves; ++octave) {
		generator_collection[size_t(octave)].GenerateNoise(noiseField, coordinate, size, p_scale * multiplier,
		                                                   multiplier, octave == firstOctave);
		multiplier /= 2.0;
	}
}

inline void NoiseOctavesPerlin::GenerateOctaves(std::vector<double>& noiseField, Int32_2 offset, Int32_2 size,
                                                Vec2 scale, [[maybe_unused]] double unused) {
	this->GenerateOctaves(noiseField, Vec3{ double(offset.x), 10.0, double(offset.z) }, Int32_3{ size.x, 1, size.z },
	                      Vec3{ scale.x, 1.0, scale.z });
}