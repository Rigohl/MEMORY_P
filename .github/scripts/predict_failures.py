#!/usr/bin/env python3
"""
Predictive Failure Analysis for MEMORY_P CI/CD

This script analyzes historical CI/CD data to predict potential failures
using statistical analysis and pattern detection.

Features:
- Failure rate analysis
- Duration trend detection
- Branch-specific failure patterns
- Retry pattern analysis
- Risk scoring and alerting
"""

import json
import sys
from datetime import datetime, timedelta
from typing import Dict, List, Any
from dataclasses import dataclass


@dataclass
class Prediction:
    """Prediction result with risk assessment"""
    risk_level: str  # 'low', 'medium', 'high', 'critical'
    confidence: float  # 0.0 to 100.0
    patterns_detected: List[str]
    recommendations: List[str]
    metrics: Dict[str, Any]


class FailurePredictor:
    """Predicts CI/CD failures based on historical data"""
    
    # Risk thresholds
    HIGH_FAILURE_RATE = 0.30
    MEDIUM_FAILURE_RATE = 0.15
    CRITICAL_FAILURE_RATE = 0.50
    
    SLOW_BUILD_THRESHOLD_MS = 1800000  # 30 minutes
    VERY_SLOW_BUILD_THRESHOLD_MS = 3600000  # 60 minutes
    
    BRANCH_FAILURE_THRESHOLD = 3
    CRITICAL_BRANCH_FAILURE_THRESHOLD = 5
    
    def __init__(self, metrics_data: Dict[str, Any]):
        self.metrics = metrics_data
        self.runs = metrics_data.get('workflow_runs', [])
        self.completed_runs = [r for r in self.runs if r.get('status') == 'completed']
        self.recent_runs = self.completed_runs[:20]
    
    def analyze(self) -> Prediction:
        """Perform comprehensive failure prediction analysis"""
        patterns = []
        recommendations = []
        risk_level = 'low'
        confidence = 0.0
        
        # Analyze failure rate
        failure_analysis = self._analyze_failure_rate()
        patterns.extend(failure_analysis['patterns'])
        recommendations.extend(failure_analysis['recommendations'])
        
        if failure_analysis['risk_level'] == 'critical':
            risk_level = 'critical'
            confidence = max(confidence, failure_analysis['confidence'])
        elif failure_analysis['risk_level'] == 'high' and risk_level != 'critical':
            risk_level = 'high'
            confidence = max(confidence, failure_analysis['confidence'])
        elif failure_analysis['risk_level'] == 'medium' and risk_level == 'low':
            risk_level = 'medium'
            confidence = max(confidence, failure_analysis['confidence'])
        
        # Analyze build duration
        duration_analysis = self._analyze_build_duration()
        patterns.extend(duration_analysis['patterns'])
        recommendations.extend(duration_analysis['recommendations'])
        
        # Analyze branch-specific issues
        branch_analysis = self._analyze_branch_failures()
        patterns.extend(branch_analysis['patterns'])
        recommendations.extend(branch_analysis['recommendations'])
        
        if branch_analysis['critical_branches']:
            risk_level = 'high' if risk_level != 'critical' else 'critical'
            confidence = max(confidence, 75.0)
        
        # Analyze retry patterns
        retry_analysis = self._analyze_retry_patterns()
        patterns.extend(retry_analysis['patterns'])
        recommendations.extend(retry_analysis['recommendations'])
        
        # Calculate overall metrics
        metrics = {
            'total_runs_analyzed': len(self.runs),
            'recent_runs_analyzed': len(self.recent_runs),
            'overall_failure_rate': failure_analysis['overall_rate'],
            'recent_failure_rate': failure_analysis['recent_rate'],
            'avg_duration_minutes': duration_analysis['avg_duration'],
            'critical_branches': branch_analysis['critical_branches'],
            'timestamp': datetime.utcnow().isoformat()
        }
        
        return Prediction(
            risk_level=risk_level,
            confidence=min(confidence, 95.0),  # Cap at 95%
            patterns_detected=patterns,
            recommendations=recommendations,
            metrics=metrics
        )
    
    def _analyze_failure_rate(self) -> Dict[str, Any]:
        """Analyze failure rates and trends"""
        if not self.recent_runs:
            return {
                'risk_level': 'low',
                'confidence': 0.0,
                'patterns': ['Insufficient data for failure rate analysis'],
                'recommendations': [],
                'overall_rate': 0.0,
                'recent_rate': 0.0
            }
        
        # Calculate rates
        failed_runs = [r for r in self.recent_runs if r.get('conclusion') == 'failure']
        recent_failure_rate = len(failed_runs) / len(self.recent_runs)
        
        overall_failed = [r for r in self.completed_runs if r.get('conclusion') == 'failure']
        overall_failure_rate = len(overall_failed) / len(self.completed_runs) if self.completed_runs else 0.0
        
        patterns = []
        recommendations = []
        risk_level = 'low'
        confidence = recent_failure_rate * 100
        
        if recent_failure_rate >= self.CRITICAL_FAILURE_RATE:
            risk_level = 'critical'
            confidence = min(95.0, confidence)
            patterns.append(f'🔴 CRITICAL: Recent failure rate at {recent_failure_rate*100:.1f}%')
            recommendations.append('IMMEDIATE ACTION REQUIRED: CI/CD pipeline is critically unstable')
            recommendations.append('Consider reverting recent changes or implementing emergency fixes')
        elif recent_failure_rate >= self.HIGH_FAILURE_RATE:
            risk_level = 'high'
            patterns.append(f'⚠️ High recent failure rate: {recent_failure_rate*100:.1f}%')
            recommendations.append('Review recent changes and consider reverting problematic commits')
            recommendations.append('Investigate common failure patterns in recent builds')
        elif recent_failure_rate >= self.MEDIUM_FAILURE_RATE:
            risk_level = 'medium'
            patterns.append(f'⚡ Elevated failure rate: {recent_failure_rate*100:.1f}%')
            recommendations.append('Monitor upcoming builds closely for continued issues')
        else:
            patterns.append(f'✅ Acceptable failure rate: {recent_failure_rate*100:.1f}%')
        
        # Check for increasing trend
        if len(self.recent_runs) >= 10:
            first_half = self.recent_runs[10:20]
            second_half = self.recent_runs[:10]
            
            first_half_failures = len([r for r in first_half if r.get('conclusion') == 'failure'])
            second_half_failures = len([r for r in second_half if r.get('conclusion') == 'failure'])
            
            if second_half_failures > first_half_failures * 1.5:
                patterns.append('📈 Failure rate is trending upward')
                recommendations.append('Investigate recent changes causing increased failures')
        
        return {
            'risk_level': risk_level,
            'confidence': confidence,
            'patterns': patterns,
            'recommendations': recommendations,
            'overall_rate': overall_failure_rate,
            'recent_rate': recent_failure_rate
        }
    
    def _analyze_build_duration(self) -> Dict[str, Any]:
        """Analyze build duration trends"""
        durations = [r.get('duration_ms', 0) for r in self.recent_runs if r.get('duration_ms', 0) > 0]
        
        if not durations:
            return {
                'patterns': [],
                'recommendations': [],
                'avg_duration': 0.0
            }
        
        avg_duration = sum(durations) / len(durations)
        avg_duration_minutes = avg_duration / 1000 / 60
        
        patterns = []
        recommendations = []
        
        if avg_duration >= self.VERY_SLOW_BUILD_THRESHOLD_MS:
            patterns.append(f'🐌 Very slow builds detected: {avg_duration_minutes:.1f} min average')
            recommendations.append('Critical: Build times are extremely long')
            recommendations.append('Review CI/CD pipeline optimization opportunities')
            recommendations.append('Consider parallelizing jobs or upgrading runners')
        elif avg_duration >= self.SLOW_BUILD_THRESHOLD_MS:
            patterns.append(f'⏰ Long build times detected: {avg_duration_minutes:.1f} min average')
            recommendations.append('Consider optimizing CI pipeline or increasing caching')
            recommendations.append('Review if all tests are necessary in CI')
        else:
            patterns.append(f'⚡ Build times are acceptable: {avg_duration_minutes:.1f} min average')
        
        # Check for increasing duration trend
        if len(durations) >= 10:
            recent_avg = sum(durations[:5]) / 5
            older_avg = sum(durations[5:10]) / 5
            
            if recent_avg > older_avg * 1.3:
                patterns.append('📈 Build durations are trending upward')
                recommendations.append('Investigate recent changes impacting build performance')
        
        return {
            'patterns': patterns,
            'recommendations': recommendations,
            'avg_duration': avg_duration_minutes
        }
    
    def _analyze_branch_failures(self) -> Dict[str, Any]:
        """Analyze branch-specific failure patterns"""
        branch_failures = {}
        branch_attempts = {}
        
        for run in self.recent_runs:
            branch = run.get('branch', 'unknown')
            branch_attempts[branch] = branch_attempts.get(branch, 0) + 1
            
            if run.get('conclusion') == 'failure':
                branch_failures[branch] = branch_failures.get(branch, 0) + 1
        
        patterns = []
        recommendations = []
        critical_branches = []
        
        for branch, failure_count in branch_failures.items():
            attempts = branch_attempts.get(branch, 1)
            failure_rate = failure_count / attempts
            
            if failure_count >= self.CRITICAL_BRANCH_FAILURE_THRESHOLD:
                critical_branches.append(branch)
                patterns.append(f'🔴 Branch "{branch}": {failure_count} failures ({failure_rate*100:.0f}% rate)')
                recommendations.append(f'CRITICAL: Investigate branch "{branch}" - multiple consecutive failures')
            elif failure_count >= self.BRANCH_FAILURE_THRESHOLD:
                patterns.append(f'⚠️ Branch "{branch}": {failure_count} recent failures ({failure_rate*100:.0f}% rate)')
                recommendations.append(f'Review changes specific to branch "{branch}"')
        
        return {
            'patterns': patterns,
            'recommendations': recommendations,
            'critical_branches': critical_branches
        }
    
    def _analyze_retry_patterns(self) -> Dict[str, Any]:
        """Analyze retry patterns indicating flaky tests or infrastructure issues"""
        retry_runs = [r for r in self.recent_runs if r.get('attempt', 1) > 1]
        
        patterns = []
        recommendations = []
        
        if len(retry_runs) > len(self.recent_runs) * 0.2:  # More than 20% retries
            patterns.append(f'🔄 High retry rate: {len(retry_runs)} of {len(self.recent_runs)} runs')
            recommendations.append('Investigate flaky tests or infrastructure instability')
            recommendations.append('Consider implementing retry logic for known flaky tests')
        
        return {
            'patterns': patterns,
            'recommendations': recommendations
        }


def main():
    """Main entry point"""
    try:
        # Load metrics data
        with open('ci-metrics.json', 'r') as f:
            metrics_data = json.load(f)
        
        # Run prediction analysis
        predictor = FailurePredictor(metrics_data)
        prediction = predictor.analyze()
        
        # Output results
        result = {
            'risk_level': prediction.risk_level,
            'confidence': prediction.confidence,
            'patterns_detected': prediction.patterns_detected,
            'recommendations': prediction.recommendations,
            'metrics': prediction.metrics
        }
        
        print(json.dumps(result, indent=2))
        
        # Write to file
        with open('predictions.json', 'w') as f:
            json.dump(result, f, indent=2)
        
        # Exit with appropriate code
        if prediction.risk_level == 'critical':
            sys.exit(2)
        elif prediction.risk_level == 'high':
            sys.exit(1)
        else:
            sys.exit(0)
    
    except Exception as e:
        print(f"Error during prediction analysis: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
